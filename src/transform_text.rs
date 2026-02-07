use crate::options::{SpaceOperation, TextChangeOptions};
use crate::parser::SpacingContext;
use crate::replacements::TextReplacement;

/// Apply text transformations based on the given options to a text string
/// Returns None if there are no changes, Some(replacement) if changes are made
#[allow(dead_code)]
pub fn apply_text_transformation(
    start: usize,
    end: usize,
    text: &str,
    options: &TextChangeOptions,
) -> Option<TextReplacement> {
    apply_text_transformation_with_context(start, end, text, options, None)
}

/// Apply text transformations using optional AST-derived spacing context.
pub fn apply_text_transformation_with_context(
    start: usize,
    end: usize,
    text: &str,
    options: &TextChangeOptions,
    context: Option<&SpacingContext>,
) -> Option<TextReplacement> {
    apply_text_changes(text, options, start, context).map(|modified| TextReplacement {
        start,
        end,
        text: modified,
    })
}

/// Helper function to determine if space should be added before a character/operator
fn should_add_space_before(
    operation: &SpaceOperation,
    prev_char: Option<char>,
    target_char: char,
) -> bool {
    match operation {
        SpaceOperation::NoChange => false,
        SpaceOperation::After => false, // Handled elsewhere
        SpaceOperation::Before => {
            if let Some(prev_ch) = prev_char {
                !prev_ch.is_whitespace() && prev_ch != target_char
            } else {
                false
            }
        }
        SpaceOperation::BeforeAndAfter => {
            if let Some(prev_ch) = prev_char {
                !prev_ch.is_whitespace() && prev_ch != target_char
            } else {
                false
            }
        }
    }
}

type CharIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

struct OperatorContext<'a, 'b, F>
where
    F: Fn(char, &mut String, &mut String),
{
    chars: &'a mut CharIter<'b>,
    prev_char: Option<char>,
    current_line: &'a mut String,
    result: &'a mut String,
    push_char: &'a F,
    do_trim: bool,
}

// Helper functions for operator handling
fn active_buf<'a>(
    do_trim: bool,
    current_line: &'a mut String,
    result: &'a mut String,
) -> &'a mut String {
    if do_trim { current_line } else { result }
}

fn remove_trailing_ws(buf: &mut String) {
    // Preserve leading indentation when an operator starts a line.
    let mut line_has_non_ws = false;
    for ch in buf.chars().rev() {
        if ch == '\n' || ch == '\r' {
            break;
        }
        if ch != ' ' && ch != '\t' {
            line_has_non_ws = true;
            break;
        }
    }

    if !line_has_non_ws {
        return;
    }

    while let Some(last) = buf.chars().last() {
        if last == ' ' || last == '\t' {
            buf.pop();
        } else {
            break;
        }
    }
}

fn ensure_one_space_before(buf: &mut String) {
    if buf.is_empty() {
        return;
    }
    if let Some(last) = buf.chars().last() {
        if last == '\n' || last == '\r' {
            return;
        }
        if last != ' ' && last != '\t' {
            buf.push(' ');
        }
    }
}

fn consume_following_ws(chars: &mut CharIter<'_>) {
    while let Some(&(_, c)) = chars.peek() {
        if c == ' ' || c == '\t' {
            chars.next();
        } else {
            break;
        }
    }
}

fn maybe_add_space_after(op: &SpaceOperation, chars: &mut CharIter<'_>, buf: &mut String) {
    match op {
        SpaceOperation::After | SpaceOperation::BeforeAndAfter => {
            if let Some((_, nc)) = chars.peek().copied()
                && !nc.is_whitespace()
            {
                buf.push(' ');
            }
        }
        _ => {}
    }
}

// Wrapper functions for specific use cases
fn one_space_before_if_needed(buf: &mut String, op_char: char) {
    if buf.is_empty() {
        return;
    }
    if let Some(last) = buf.chars().last() {
        if last == '\n' || last == '\r' {
            return;
        }
        if last == op_char {
            return;
        }
        if last != ' ' && last != '\t' {
            buf.push(' ');
        }
    }
}

fn space_after_if_needed(
    op: &SpaceOperation,
    chars: &mut CharIter<'_>,
    buf: &mut String,
    this_char: char,
) {
    match op {
        SpaceOperation::After | SpaceOperation::BeforeAndAfter => {
            // Do not add space if the next char is identical (e.g., ++, --, ==)
            if let Some((_, nc)) = chars.peek().copied()
                && !nc.is_whitespace()
                && nc != this_char
            {
                buf.push(' ');
            }
        }
        _ => {}
    }
}

/// Generic handler for two-character operators
fn handle_two_char_operator<'a, 'b, F>(
    first_char: char,
    second_char: char,
    operation: &SpaceOperation,
    ctx: &mut OperatorContext<'a, 'b, F>,
) where
    F: Fn(char, &mut String, &mut String),
{
    ctx.chars.next(); // consume the second character
    let push_char = ctx.push_char;
    match operation {
        SpaceOperation::NoChange => {
            if should_add_space_before(operation, ctx.prev_char, first_char) {
                push_char(' ', ctx.current_line, ctx.result);
            }
            push_char(first_char, ctx.current_line, ctx.result);
            push_char(second_char, ctx.current_line, ctx.result);
            if should_add_space_after(operation, ctx.chars.peek().map(|(_, ch)| *ch), second_char) {
                push_char(' ', ctx.current_line, ctx.result);
            }
        }
        _ => {
            let buf = active_buf(ctx.do_trim, ctx.current_line, ctx.result);
            remove_trailing_ws(buf);
            if matches!(
                operation,
                SpaceOperation::Before | SpaceOperation::BeforeAndAfter
            ) {
                ensure_one_space_before(buf);
            }
            push_char(first_char, ctx.current_line, ctx.result);
            push_char(second_char, ctx.current_line, ctx.result);
            consume_following_ws(ctx.chars);
            let buf = active_buf(ctx.do_trim, ctx.current_line, ctx.result);
            maybe_add_space_after(operation, ctx.chars, buf);
        }
    }
}

/// Helper function to handle multi-character operators
fn handle_operator<'a, 'b, F>(
    current_char: char,
    operation: &SpaceOperation,
    ctx: &mut OperatorContext<'a, 'b, F>,
) -> Option<String>
where
    F: Fn(char, &mut String, &mut String),
{
    // Check for multi-character operators starting with current_char
    let next_char = ctx.chars.peek().map(|(_, ch)| *ch);

    match (current_char, next_char) {
        // Two-character operators
        ('<', Some('=')) => {
            handle_two_char_operator('<', '=', operation, ctx);
            Some("<=".to_string())
        }
        ('<', Some('>')) => {
            handle_two_char_operator('<', '>', operation, ctx);
            Some("<>".to_string())
        }
        ('>', Some('=')) => {
            handle_two_char_operator('>', '=', operation, ctx);
            Some(">=".to_string())
        }
        (':', Some('=')) => {
            handle_two_char_operator(':', '=', operation, ctx);
            Some(":=".to_string())
        }
        ('+', Some('=')) => {
            handle_two_char_operator('+', '=', operation, ctx);
            Some("+=".to_string())
        }
        ('-', Some('=')) => {
            handle_two_char_operator('-', '=', operation, ctx);
            Some("-=".to_string())
        }
        ('*', Some('=')) => {
            handle_two_char_operator('*', '=', operation, ctx);
            Some("*=".to_string())
        }
        ('/', Some('=')) => {
            handle_two_char_operator('/', '=', operation, ctx);
            Some("/=".to_string())
        }
        _ => None, // Not a multi-character operator
    }
}

/// Helper function to determine if space should be added after a character
fn should_add_space_after(
    operation: &SpaceOperation,
    next_char: Option<char>,
    target_char: char,
) -> bool {
    match operation {
        SpaceOperation::NoChange => false,
        SpaceOperation::After => {
            if let Some(next_ch) = next_char {
                !next_ch.is_whitespace() && next_ch != target_char
            } else {
                false
            }
        }
        SpaceOperation::Before => false, // Handled elsewhere
        SpaceOperation::BeforeAndAfter => {
            if let Some(next_ch) = next_char {
                !next_ch.is_whitespace() && next_ch != target_char
            } else {
                false
            }
        }
    }
}

/// Helper function to check if a character is numeric (digit)
fn is_numeric_char(ch: char) -> bool {
    ch.is_ascii_digit()
}

/// Helper function to check if colon spacing should be skipped due to numeric exception
fn should_skip_colon_spacing(
    enable_exception: bool,
    prev_char: Option<char>,
    next_char: Option<char>,
) -> bool {
    if !enable_exception {
        return false;
    }

    match (prev_char, next_char) {
        (Some(prev), Some(next)) => is_numeric_char(prev) && is_numeric_char(next),
        _ => false,
    }
}

fn is_negative_literal_minus(context: Option<&SpacingContext>, abs_pos: usize) -> bool {
    context.is_some_and(|ctx| ctx.negative_literal_minus_positions.contains(&abs_pos))
}

fn is_unary_minus(context: Option<&SpacingContext>, abs_pos: usize) -> bool {
    context.is_some_and(|ctx| ctx.unary_minus_positions.contains(&abs_pos))
}

fn is_unary_plus(context: Option<&SpacingContext>, abs_pos: usize) -> bool {
    context.is_some_and(|ctx| ctx.unary_plus_positions.contains(&abs_pos))
}

fn is_positive_literal_plus(context: Option<&SpacingContext>, abs_pos: usize) -> bool {
    context.is_some_and(|ctx| ctx.positive_literal_plus_positions.contains(&abs_pos))
}

fn is_exponent_sign(context: Option<&SpacingContext>, abs_pos: usize) -> bool {
    context.is_some_and(|ctx| ctx.exponent_sign_positions.contains(&abs_pos))
}

fn is_exponent_sign_lexical(text: &str, idx: usize) -> bool {
    let bytes = text.as_bytes();
    if idx >= bytes.len() {
        return false;
    }
    let sign = bytes[idx];
    if sign != b'+' && sign != b'-' {
        return false;
    }
    if idx == 0 {
        return false;
    }
    let e = bytes[idx - 1];
    if e != b'E' && e != b'e' {
        return false;
    }
    if idx < 2 {
        return false;
    }
    let mut saw_digit = false;
    let mut j: isize = idx as isize - 2;
    while j >= 0 {
        let b = bytes[j as usize];
        if b.is_ascii_digit() {
            saw_digit = true;
        } else if b != b'.' {
            break;
        }
        j -= 1;
    }
    if !saw_digit {
        return false;
    }
    if j >= 0 {
        let b = bytes[j as usize];
        if b.is_ascii_alphabetic() || b == b'_' {
            return false;
        }
    }
    true
}

fn is_generic_angle(context: Option<&SpacingContext>, abs_pos: usize) -> bool {
    context.is_some_and(|ctx| ctx.generic_angle_positions.contains(&abs_pos))
}

/// Apply all text changes to a text string based on the given options
fn apply_text_changes(
    text: &str,
    options: &TextChangeOptions,
    start_offset: usize,
    context: Option<&SpacingContext>,
) -> Option<String> {
    // State machine to skip Delphi string literals and comments for spacing insertion.
    // We still may trim trailing whitespace (optionally) per line, but trimming is safe
    // inside comments / strings per spec given by user.
    #[derive(Copy, Clone, PartialEq)]
    enum State {
        Code,
        StringLiteral,    // Inside '...'
        LineComment,      // // until newline
        BraceComment,     // { ... }
        ParenStarComment, // (* ... *)
    }

    let mut result = String::with_capacity(text.len());
    let mut state = State::Code;
    let mut chars = text.char_indices().peekable();
    let mut prev_char: Option<char> = None;

    // For trimming we accumulate current line raw output, then on newline flush trimmed.
    let do_trim = options.trim_trailing_whitespace;
    let mut current_line = String::new();

    // Helper to push a character to either current line buffer (if trimming) or directly.
    let push_char = |c: char, current_line: &mut String, result: &mut String| {
        if do_trim {
            current_line.push(c);
        } else {
            result.push(c);
        }
    };

    // Helper to flush a newline (\n or \r) handling trimming.
    let flush_line_ending = |newline: char, current_line: &mut String, result: &mut String| {
        if do_trim {
            // Trim end whitespace of accumulated line, then push
            let trimmed = current_line.trim_end();
            result.push_str(trimmed);
            current_line.clear();
            result.push(newline);
        } else {
            result.push(newline);
        }
    };

    while let Some((idx, ch)) = chars.next() {
        let abs_pos = start_offset + idx;
        match state {
            State::Code => {
                match ch {
                    '\'' => {
                        // Enter string literal
                        push_char(ch, &mut current_line, &mut result);
                        state = State::StringLiteral;
                    }
                    '{' => {
                        // Brace comment
                        push_char(ch, &mut current_line, &mut result);
                        state = State::BraceComment;
                    }
                    '(' => {
                        // Could start (* comment *)
                        if let Some((_, '*')) = chars.peek().copied() {
                            // consume '*'
                            let (_, star) = chars.next().unwrap();
                            push_char('(', &mut current_line, &mut result);
                            push_char(star, &mut current_line, &mut result);
                            state = State::ParenStarComment;
                        } else {
                            push_char('(', &mut current_line, &mut result);
                        }
                    }
                    '/' => {
                        if let Some((_, '/')) = chars.peek().copied() {
                            // line comment
                            let (_, slash2) = chars.next().unwrap();
                            push_char('/', &mut current_line, &mut result);
                            push_char(slash2, &mut current_line, &mut result);
                            state = State::LineComment;
                        } else if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.assign_div, &mut ctx)
                        } {
                            // '/=' handled by handle_operator
                        } else {
                            match options.fdiv {
                                SpaceOperation::NoChange => {
                                    if should_add_space_before(&options.fdiv, prev_char, '/') {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char('/', &mut current_line, &mut result);
                                    if should_add_space_after(
                                        &options.fdiv,
                                        chars.peek().map(|(_, ch)| *ch),
                                        '/',
                                    ) {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if matches!(
                                        op,
                                        SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                    ) {
                                        one_space_before_if_needed(buf, '/');
                                    }
                                    push_char('/', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    space_after_if_needed(op, &mut chars, buf, '/');
                                }
                            }
                        }
                    }
                    ',' => {
                        match options.comma {
                            SpaceOperation::NoChange => {
                                if should_add_space_before(&options.comma, prev_char, ',') {
                                    push_char(' ', &mut current_line, &mut result);
                                }
                                push_char(',', &mut current_line, &mut result);
                                if should_add_space_after(
                                    &options.comma,
                                    chars.peek().map(|(_, ch)| *ch),
                                    ',',
                                ) {
                                    push_char(' ', &mut current_line, &mut result);
                                }
                            }
                            ref op => {
                                let buf = if do_trim {
                                    &mut current_line
                                } else {
                                    &mut result
                                };
                                remove_trailing_ws(buf);
                                if matches!(
                                    op,
                                    SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                ) {
                                    one_space_before_if_needed(buf, ',');
                                }
                                push_char(',', &mut current_line, &mut result);
                                consume_following_ws(&mut chars);
                                let buf = if do_trim {
                                    &mut current_line
                                } else {
                                    &mut result
                                };
                                // For comma: only add space if next char is not punctuation we purposely keep adjacent (semicolon)
                                if let Some((_, nc)) = chars.peek().copied() {
                                    if nc == ';' {
                                        // We still want exactly one space after comma before semicolon if comma rule demands After
                                        if matches!(
                                            op,
                                            SpaceOperation::After | SpaceOperation::BeforeAndAfter
                                        ) {
                                            buf.push(' ');
                                        }
                                    } else {
                                        space_after_if_needed(op, &mut chars, buf, ',');
                                    }
                                } else {
                                    space_after_if_needed(op, &mut chars, buf, ',');
                                }
                            }
                        }
                    }
                    ';' => match options.semi_colon {
                        SpaceOperation::NoChange => {
                            if should_add_space_before(&options.semi_colon, prev_char, ';') {
                                push_char(' ', &mut current_line, &mut result);
                            }
                            push_char(';', &mut current_line, &mut result);
                            if should_add_space_after(
                                &options.semi_colon,
                                chars.peek().map(|(_, ch)| *ch),
                                ';',
                            ) {
                                push_char(' ', &mut current_line, &mut result);
                            }
                        }
                        ref op => {
                            let buf = if do_trim {
                                &mut current_line
                            } else {
                                &mut result
                            };
                            remove_trailing_ws(buf);
                            if matches!(op, SpaceOperation::Before | SpaceOperation::BeforeAndAfter)
                            {
                                one_space_before_if_needed(buf, ';');
                            }
                            push_char(';', &mut current_line, &mut result);
                            consume_following_ws(&mut chars);
                            let buf = if do_trim {
                                &mut current_line
                            } else {
                                &mut result
                            };
                            space_after_if_needed(op, &mut chars, buf, ';');
                        }
                    },
                    '<' => {
                        if is_generic_angle(context, abs_pos) {
                            let buf = active_buf(do_trim, &mut current_line, &mut result);
                            remove_trailing_ws(buf);
                            push_char('<', &mut current_line, &mut result);
                            consume_following_ws(&mut chars);
                        } else if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.lte, &mut ctx)
                        } {
                            // '<=' handled by handle_operator
                        } else if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.neq, &mut ctx)
                        } {
                            // '<>' handled by handle_operator
                        } else {
                            match options.lt {
                                SpaceOperation::NoChange => {
                                    if should_add_space_before(&options.lt, prev_char, '<') {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char('<', &mut current_line, &mut result);
                                    if should_add_space_after(
                                        &options.lt,
                                        chars.peek().map(|(_, ch)| *ch),
                                        '<',
                                    ) {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if matches!(
                                        op,
                                        SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                    ) {
                                        one_space_before_if_needed(buf, '<');
                                    }
                                    push_char('<', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    space_after_if_needed(op, &mut chars, buf, '<');
                                }
                            }
                        }
                    }
                    '=' => match options.eq {
                        SpaceOperation::NoChange => {
                            if should_add_space_before(&options.eq, prev_char, '=') {
                                push_char(' ', &mut current_line, &mut result);
                            }
                            push_char('=', &mut current_line, &mut result);
                            if should_add_space_after(
                                &options.eq,
                                chars.peek().map(|(_, ch)| *ch),
                                '=',
                            ) {
                                push_char(' ', &mut current_line, &mut result);
                            }
                        }
                        ref op => {
                            let buf = if do_trim {
                                &mut current_line
                            } else {
                                &mut result
                            };
                            remove_trailing_ws(buf);
                            if matches!(op, SpaceOperation::Before | SpaceOperation::BeforeAndAfter)
                            {
                                one_space_before_if_needed(buf, '=');
                            }
                            push_char('=', &mut current_line, &mut result);
                            consume_following_ws(&mut chars);
                            let buf = if do_trim {
                                &mut current_line
                            } else {
                                &mut result
                            };
                            space_after_if_needed(op, &mut chars, buf, '=');
                        }
                    },
                    '>' => {
                        if is_generic_angle(context, abs_pos) {
                            let buf = active_buf(do_trim, &mut current_line, &mut result);
                            remove_trailing_ws(buf);
                            push_char('>', &mut current_line, &mut result);
                            consume_following_ws(&mut chars);
                        } else if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.gte, &mut ctx)
                        } {
                            // '>=' handled by handle_operator
                        } else {
                            match options.gt {
                                SpaceOperation::NoChange => {
                                    if should_add_space_before(&options.gt, prev_char, '>') {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char('>', &mut current_line, &mut result);
                                    if should_add_space_after(
                                        &options.gt,
                                        chars.peek().map(|(_, ch)| *ch),
                                        '>',
                                    ) {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if matches!(
                                        op,
                                        SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                    ) {
                                        one_space_before_if_needed(buf, '>');
                                    }
                                    push_char('>', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    space_after_if_needed(op, &mut chars, buf, '>');
                                }
                            }
                        }
                    }
                    '+' => {
                        if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.assign_add, &mut ctx)
                        } {
                            // '+=' handled by handle_operator
                        } else if is_unary_plus(context, abs_pos)
                            || is_positive_literal_plus(context, abs_pos)
                            || is_exponent_sign(context, abs_pos)
                            || is_exponent_sign_lexical(text, idx)
                        {
                            push_char('+', &mut current_line, &mut result);
                            consume_following_ws(&mut chars);
                        } else {
                            match options.add {
                                SpaceOperation::NoChange => {
                                    if should_add_space_before(&options.add, prev_char, '+') {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char('+', &mut current_line, &mut result);
                                    if should_add_space_after(
                                        &options.add,
                                        chars.peek().map(|(_, ch)| *ch),
                                        '+',
                                    ) {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if matches!(
                                        op,
                                        SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                    ) {
                                        one_space_before_if_needed(buf, '+');
                                    }
                                    push_char('+', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    space_after_if_needed(op, &mut chars, buf, '+');
                                }
                            }
                        }
                    }
                    '-' => {
                        if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.assign_sub, &mut ctx)
                        } {
                            // '-=' handled by handle_operator
                        } else if is_negative_literal_minus(context, abs_pos)
                            || is_unary_minus(context, abs_pos)
                            || is_exponent_sign(context, abs_pos)
                            || is_exponent_sign_lexical(text, idx)
                        {
                            push_char('-', &mut current_line, &mut result);
                            consume_following_ws(&mut chars);
                        } else {
                            match options.sub {
                                SpaceOperation::NoChange => {
                                    if should_add_space_before(&options.sub, prev_char, '-') {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char('-', &mut current_line, &mut result);
                                    if should_add_space_after(
                                        &options.sub,
                                        chars.peek().map(|(_, ch)| *ch),
                                        '-',
                                    ) {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if matches!(
                                        op,
                                        SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                    ) {
                                        one_space_before_if_needed(buf, '-');
                                    }
                                    push_char('-', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    space_after_if_needed(op, &mut chars, buf, '-');
                                }
                            }
                        }
                    }
                    '*' => {
                        if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.assign_mul, &mut ctx)
                        } {
                            // '*=' handled by handle_operator
                        } else {
                            match options.mul {
                                SpaceOperation::NoChange => {
                                    if should_add_space_before(&options.mul, prev_char, '*') {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char('*', &mut current_line, &mut result);
                                    if should_add_space_after(
                                        &options.mul,
                                        chars.peek().map(|(_, ch)| *ch),
                                        '*',
                                    ) {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if matches!(
                                        op,
                                        SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                    ) {
                                        one_space_before_if_needed(buf, '*');
                                    }
                                    push_char('*', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    space_after_if_needed(op, &mut chars, buf, '*');
                                }
                            }
                        }
                    }
                    ':' => {
                        if let Some(_handled) = {
                            let mut ctx = OperatorContext {
                                chars: &mut chars,
                                prev_char,
                                current_line: &mut current_line,
                                result: &mut result,
                                push_char: &push_char,
                                do_trim,
                            };
                            handle_operator(ch, &options.assign, &mut ctx)
                        } {
                            // ':=' handled by handle_operator
                        } else {
                            // Single ':' operator
                            // Check if we should skip spacing due to numeric exception (e.g., time format like "12:34")
                            let skip_spacing = should_skip_colon_spacing(
                                options.colon_numeric_exception,
                                prev_char,
                                chars.peek().map(|(_, ch)| *ch),
                            );
                            match options.colon {
                                SpaceOperation::NoChange => {
                                    if !skip_spacing
                                        && should_add_space_before(&options.colon, prev_char, ':')
                                    {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                    push_char(':', &mut current_line, &mut result);
                                    if !skip_spacing
                                        && should_add_space_after(
                                            &options.colon,
                                            chars.peek().map(|(_, ch)| *ch),
                                            ':',
                                        )
                                    {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                                ref op => {
                                    let buf = if do_trim {
                                        &mut current_line
                                    } else {
                                        &mut result
                                    };
                                    remove_trailing_ws(buf);
                                    if !skip_spacing
                                        && matches!(
                                            op,
                                            SpaceOperation::Before | SpaceOperation::BeforeAndAfter
                                        )
                                    {
                                        one_space_before_if_needed(buf, ':');
                                    }
                                    push_char(':', &mut current_line, &mut result);
                                    consume_following_ws(&mut chars);
                                    if !skip_spacing
                                        && matches!(
                                            op,
                                            SpaceOperation::After | SpaceOperation::BeforeAndAfter
                                        )
                                        && let Some((_, nc)) = chars.peek().copied()
                                        && !nc.is_whitespace()
                                        && nc != ':'
                                    {
                                        push_char(' ', &mut current_line, &mut result);
                                    }
                                }
                            }
                        }
                    }
                    '\n' | '\r' => {
                        flush_line_ending(ch, &mut current_line, &mut result);
                    }
                    _ => {
                        push_char(ch, &mut current_line, &mut result);
                    }
                }
            }
            State::StringLiteral => {
                if ch == '\n' || ch == '\r' {
                    // Unterminated string at line break: exit string state
                    flush_line_ending(ch, &mut current_line, &mut result);
                    state = State::Code;
                } else {
                    push_char(ch, &mut current_line, &mut result);
                    if ch == '\'' {
                        // Delphi/Pascal doubles '' inside a string to escape a single quote.
                        if let Some((_, '\'')) = chars.peek().copied() {
                            // This is an escaped quote, consume the second quote and stay in string
                            let (_, escaped_quote) = chars.next().unwrap();
                            push_char(escaped_quote, &mut current_line, &mut result);
                            // Stay in StringLiteral state - this is still part of the string
                        } else {
                            // End of string literal
                            state = State::Code;
                        }
                    }
                }
            }
            State::LineComment => {
                if ch == '\n' || ch == '\r' {
                    // End of line comment - use consistent flush_line_ending logic
                    flush_line_ending(ch, &mut current_line, &mut result);
                    state = State::Code;
                } else {
                    push_char(ch, &mut current_line, &mut result);
                }
            }
            State::BraceComment => {
                if ch == '\n' || ch == '\r' {
                    // Handle newlines in brace comments consistently
                    flush_line_ending(ch, &mut current_line, &mut result);
                } else {
                    push_char(ch, &mut current_line, &mut result);
                    if ch == '}' {
                        state = State::Code;
                    }
                }
            }
            State::ParenStarComment => {
                if ch == '\n' || ch == '\r' {
                    // Handle newlines in paren-star comments consistently
                    flush_line_ending(ch, &mut current_line, &mut result);
                } else {
                    push_char(ch, &mut current_line, &mut result);
                    if ch == '*' {
                        // Look ahead for ) to end comment
                        if let Some((_, ')')) = chars.peek().copied() {
                            let (_, closing_paren) = chars.next().unwrap();
                            push_char(closing_paren, &mut current_line, &mut result);
                            state = State::Code;
                        }
                    }
                }
            }
        }

        // Update previous character for next iteration
        prev_char = Some(ch);
    }

    if do_trim && !current_line.is_empty() {
        // flush last line (no newline present)
        let trimmed = current_line.trim_end();
        result.push_str(trimmed);
    }
    if result == text { None } else { Some(result) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_text_transformation_comma_only_with_identity_replacement() {
        let text = "Hello,World"; // Text to transform
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        let result = apply_text_transformation(0, 11, text, &options);
        assert_eq!(result.unwrap().text, "Hello, World".to_string());
    }

    #[test]
    fn test_apply_text_transformation_comma_only_with_regular_replacement() {
        let replacement = TextReplacement {
            start: 0,
            end: 8,
            text: "A,B,C".to_string(),
        };
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        let result = apply_text_transformation(
            replacement.start,
            replacement.end,
            replacement.text.as_str(),
            &options,
        );
        assert_eq!(result.unwrap().text, "A, B, C".to_string());
    }

    #[test]
    fn test_apply_text_transformation_mixed_replacements() {
        let source = "Hello,World and Foo,Bar";
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        // Test identity replacement
        // Identity region (no pre-existing replacement object needed)
        let text1 = &source[0..11];
        let result1 = apply_text_transformation(0, 11, text1, &options);
        assert_eq!(result1.unwrap().text, "Hello, World".to_string());

        // Test regular replacement without commas
        let replacement2 = TextReplacement {
            start: 11,
            end: 15,
            text: " and ".to_string(),
        };
        let result2 = apply_text_transformation(
            replacement2.start,
            replacement2.end,
            replacement2.text.as_str(),
            &options,
        );
        assert!(result2.is_none()); // No changes should be made

        // Test regular replacement with comma
        let replacement3 = TextReplacement {
            start: 15,
            end: 23,
            text: "Baz,Qux".to_string(),
        };
        let result3 = apply_text_transformation(
            replacement3.start,
            replacement3.end,
            replacement3.text.as_str(),
            &options,
        );
        assert_eq!(result3.unwrap().text, "Baz, Qux".to_string());
    }

    #[test]
    fn test_apply_text_transformation_uses_content() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        // Test replacement with uses content
        let uses_replacement = TextReplacement {
            start: 0,
            end: 11,
            text: "uses,System".to_string(),
        };
        let result1 = apply_text_transformation(
            uses_replacement.start,
            uses_replacement.end,
            uses_replacement.text.as_str(),
            &options,
        );
        // The function should transform it
        assert_eq!(result1.unwrap().text, "uses, System".to_string());

        // Test regular replacement
        let regular_replacement = TextReplacement {
            start: 11,
            end: 23,
            text: " test,code".to_string(),
        };
        let result2 = apply_text_transformation(
            regular_replacement.start,
            regular_replacement.end,
            regular_replacement.text.as_str(),
            &options,
        );
        assert_eq!(result2.unwrap().text, " test, code".to_string());
    }

    #[test]
    fn test_apply_text_changes_comma_only() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a, b;c, d");
    }

    #[test]
    fn test_apply_text_changes_semicolon_only() {
        let options = TextChangeOptions {
            comma: SpaceOperation::NoChange,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a,b; c,d");
    }

    #[test]
    fn test_apply_text_changes_both_comma_and_semicolon() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a, b; c, d");
    }

    #[test]
    fn test_apply_text_changes_neither() {
        let options = TextChangeOptions {
            comma: SpaceOperation::NoChange,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options, 0, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_text_transformation_with_options() {
        let replacement = TextReplacement {
            start: 0,
            end: 8,
            text: "a,b;c".to_string(),
        };
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        let result = apply_text_transformation(
            replacement.start,
            replacement.end,
            replacement.text.as_str(),
            &options,
        );
        assert_eq!(result.unwrap().text, "a, b; c".to_string());
    }

    #[test]
    fn test_apply_text_transformation_identity_replacement() {
        let source = "a,b;c";
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = &source[0..5];
        let result = apply_text_transformation(0, 5, text, &options);
        assert_eq!(result.unwrap().text, "a, b; c".to_string());
    }

    #[test]
    fn test_apply_text_transformation_regular_replacement() {
        let replacement = TextReplacement {
            start: 0,
            end: 8,
            text: "a,b;c".to_string(),
        };
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        let result = apply_text_transformation(
            replacement.start,
            replacement.end,
            replacement.text.as_str(),
            &options,
        );
        assert_eq!(result.unwrap().text, "a, b; c".to_string());
    }

    #[test]
    fn test_apply_text_changes_with_trim_trailing_whitespace() {
        let options = TextChangeOptions {
            comma: SpaceOperation::NoChange,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: true,
            ..Default::default()
        };
        let text = "Line 1   \nLine 2\t\t\nLine 3 ";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_apply_text_changes_combined_comma_and_trim() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: true,
            ..Default::default()
        };
        let text = "a,b,c   \nd,e,f\t\t";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a, b, c\nd, e, f");
    }

    #[test]
    fn test_apply_text_changes_all_options_enabled() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: true,
            ..Default::default()
        };
        let text = "a,b;c,d   \ne,f;g,h\t\t";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a, b; c, d\ne, f; g, h");
    }

    #[test]
    fn test_apply_text_transformation_with_trim_trailing_whitespace() {
        let replacement = TextReplacement {
            start: 0,
            end: 11,
            text: "a,b;c   \nd,e;f\t\t".to_string(),
        };
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: true,
            ..Default::default()
        };

        let result = apply_text_transformation(
            replacement.start,
            replacement.end,
            replacement.text.as_str(),
            &options,
        );
        assert_eq!(result.unwrap().text, "a, b; c\nd, e; f".to_string());
    }

    #[test]
    fn test_apply_text_transformation_identity_with_trim() {
        let source = "Hello,World   \nFoo;Bar\t\t";
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: true,
            ..Default::default()
        };
        let text = &source[..];
        let result = apply_text_transformation(0, source.len(), text, &options);
        assert_eq!(result.unwrap().text, "Hello, World\nFoo; Bar".to_string());
    }

    #[test]
    fn test_apply_text_transformation_no_changes() {
        let source = "Hello, World";
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = &source[..];
        let result = apply_text_transformation(0, 12, text, &options);
        assert!(result.is_none()); // No changes needed
    }

    #[test]
    fn test_apply_text_transformation_regular_replacement_no_changes() {
        let replacement = TextReplacement {
            start: 0,
            end: 8,
            text: "Hello, World".to_string(),
        }; // Already properly formatted
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };

        let result = apply_text_transformation(
            replacement.start,
            replacement.end,
            replacement.text.as_str(),
            &options,
        );
        assert!(result.is_none()); // No changes needed
    }

    // --- Tests for edge cases and bug fixes ---

    #[test]
    fn test_escaped_quotes_in_string_literals() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Test escaped single quotes in Delphi/Pascal strings
        let text = "s := 'It''s a test',x;y";
        let result = apply_text_changes(text, &options, 0, None);
        // The comma/semicolon inside the string should not be spaced
        assert_eq!(result.unwrap(), "s := 'It''s a test', x; y");
    }

    #[test]
    fn test_complex_escaped_quotes() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Multiple escaped quotes and code after
        let text = "msg := 'Can''t say ''hello'', sorry',next";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(
            result.unwrap(),
            "msg := 'Can''t say ''hello'', sorry', next"
        );
    }

    #[test]
    fn test_unterminated_string_with_line_break() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Unterminated string that breaks at newline
        let text = "s := 'unterminated\ncode,after;break";
        let result = apply_text_changes(text, &options, 0, None);
        // After line break, spacing should be applied
        assert_eq!(result.unwrap(), "s := 'unterminated\ncode, after; break");
    }

    #[test]
    fn test_multiline_comments_with_spacing() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Test multiline brace comments
        let text = "{ multi\nline,comment;here }\ncode,after";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "{ multi\nline,comment;here }\ncode, after");
    }

    #[test]
    fn test_multiline_paren_star_comments() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Test multiline (* *) comments
        let text = "(* multi\nline,comment;here *)\ncode,after";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(
            result.unwrap(),
            "(* multi\nline,comment;here *)\ncode, after"
        );
    }

    #[test]
    fn test_trim_with_different_line_endings() {
        let options = TextChangeOptions {
            comma: SpaceOperation::NoChange,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: true,
            ..Default::default()
        };
        // Test trimming with both LF and CRLF
        let text = "line1   \r\nline2\t\t\nline3   ";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "line1\r\nline2\nline3");
    }

    // --- Original tests ensuring spacing is skipped inside strings & comments ---
    #[test]
    fn test_skip_spacing_inside_string_literal() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "'a,b;c',x;y";
        // Only commas/semicolons outside the quotes should be spaced.
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "'a,b;c', x; y");
    }

    #[test]
    fn test_skip_spacing_inside_brace_comment() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "{a,b;c},x;y";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "{a,b;c}, x; y");
    }

    #[test]
    fn test_skip_spacing_inside_paren_star_comment() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "(*a,b;c*),x;y";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "(*a,b;c*), x; y");
    }

    #[test]
    fn test_skip_spacing_inside_line_comment() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "// a,b;c\nx,y;z";
        let result = apply_text_changes(text, &options, 0, None);
        // Only second line is transformed.
        assert_eq!(result.unwrap(), "// a,b;c\nx, y; z");
    }

    #[test]
    fn test_mixed_code_and_comments_and_strings() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "val:='a,b'; // c,d;e\n{ x,y;z } foo,bar;baz (* p,q;r *) qux,quux";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(
            result.unwrap(),
            "val := 'a,b'; // c,d;e\n{ x,y;z } foo, bar; baz (* p,q;r *) qux, quux"
        );
    }

    // Tests for new SpaceOperation variants
    #[test]
    fn test_space_before_comma() {
        let options = TextChangeOptions {
            comma: SpaceOperation::Before,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b,c";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a ,b ,c");
    }

    #[test]
    fn test_space_before_semicolon() {
        let options = TextChangeOptions {
            comma: SpaceOperation::NoChange,
            semi_colon: SpaceOperation::Before,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a;b;c";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a ;b ;c");
    }

    #[test]
    fn test_space_before_and_after_comma() {
        let options = TextChangeOptions {
            comma: SpaceOperation::BeforeAndAfter,
            semi_colon: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b,c";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a , b , c");
    }

    #[test]
    fn test_space_before_and_after_semicolon() {
        let options = TextChangeOptions {
            comma: SpaceOperation::NoChange,
            semi_colon: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a;b;c";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a ; b ; c");
    }

    #[test]
    fn test_space_before_doesnt_add_duplicate_space() {
        let options = TextChangeOptions {
            comma: SpaceOperation::Before,
            semi_colon: SpaceOperation::Before,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Already has spaces before punctuation - should not add more
        let text = "a ,b ;c";
        let result = apply_text_changes(text, &options, 0, None);
        assert!(result.is_none()); // No change because space already exists
    }

    #[test]
    fn test_space_after_doesnt_add_duplicate_space() {
        let options = TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Already has spaces after punctuation - should not add more
        let text = "a, b; c";
        let result = apply_text_changes(text, &options, 0, None);
        assert!(result.is_none()); // No change because space already exists
    }

    #[test]
    fn test_no_space_at_beginning_for_before_operation() {
        let options = TextChangeOptions {
            comma: SpaceOperation::Before,
            semi_colon: SpaceOperation::Before,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Comma/semicolon at the beginning should not add space before
        let text = ",a;b";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), ",a ;b"); // No space before first comma
    }

    #[test]
    fn test_mixed_space_operations() {
        let options = TextChangeOptions {
            comma: SpaceOperation::Before,
            semi_colon: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a ,b ; c ,d");
    }

    // Tests for new operators
    #[test]
    fn test_assignment_operators() {
        let options = TextChangeOptions {
            assign: SpaceOperation::BeforeAndAfter,
            assign_add: SpaceOperation::BeforeAndAfter,
            assign_sub: SpaceOperation::BeforeAndAfter,
            assign_mul: SpaceOperation::BeforeAndAfter,
            assign_div: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a:=5+b+=c-=d*=e/=f";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "a := 5 + b += c -= d *= e /= f");
    }

    #[test]
    fn test_comparison_operators() {
        let options = TextChangeOptions {
            lt: SpaceOperation::BeforeAndAfter,
            eq: SpaceOperation::BeforeAndAfter,
            neq: SpaceOperation::BeforeAndAfter,
            gt: SpaceOperation::BeforeAndAfter,
            lte: SpaceOperation::BeforeAndAfter,
            gte: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "if a<b=c<>d>e<=f>=g then";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "if a < b = c <> d > e <= f >= g then");
    }

    #[test]
    fn test_arithmetic_operators() {
        let options = TextChangeOptions {
            add: SpaceOperation::BeforeAndAfter,
            sub: SpaceOperation::BeforeAndAfter,
            mul: SpaceOperation::BeforeAndAfter,
            fdiv: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "result:=a+b-c*d/e";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "result := a + b - c * d / e");
    }

    #[test]
    fn test_colon_operator() {
        let options = TextChangeOptions {
            colon: SpaceOperation::After,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "var x:Integer;y:String;z:Boolean";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "var x: Integer; y: String; z: Boolean");
    }

    #[test]
    fn test_no_change_operators() {
        let options = TextChangeOptions {
            add: SpaceOperation::NoChange,
            sub: SpaceOperation::NoChange,
            mul: SpaceOperation::NoChange,
            fdiv: SpaceOperation::NoChange,
            eq: SpaceOperation::NoChange,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a+b-c*d/e=f";
        let result = apply_text_changes(text, &options, 0, None);
        assert!(result.is_none()); // Should remain unchanged for these operators
    }

    #[test]
    fn test_operators_with_comments_and_strings() {
        let options = TextChangeOptions {
            assign: SpaceOperation::BeforeAndAfter,
            eq: SpaceOperation::BeforeAndAfter,
            add: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "msg:='a:=b+c'; // Comment with := and + and =\nresult:=x=y+z";
        let result = apply_text_changes(text, &options, 0, None);
        // Operators inside string and comments should not be spaced
        assert_eq!(
            result.unwrap(),
            "msg := 'a:=b+c'; // Comment with := and + and =\nresult := x = y + z"
        );
    }

    #[test]
    fn test_consecutive_operators() {
        let options = TextChangeOptions {
            add: SpaceOperation::BeforeAndAfter,
            sub: SpaceOperation::BeforeAndAfter,
            eq: SpaceOperation::BeforeAndAfter,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        let text = "a++b--c==d";
        let result = apply_text_changes(text, &options, 0, None);
        // Consecutive same operators should not have space between them (correct behavior)
        assert_eq!(result.unwrap(), "a ++ b -- c == d");
    }

    // Tests for colon numeric exception
    #[test]
    fn test_colon_numeric_exception_enabled() {
        let options = TextChangeOptions {
            colon: SpaceOperation::BeforeAndAfter,
            colon_numeric_exception: true,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Time format - should not have spaces when numeric exception is enabled
        let text = "time := 12:34:56;";
        let result = apply_text_changes(text, &options, 0, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_colon_numeric_exception_disabled() {
        let options = TextChangeOptions {
            colon: SpaceOperation::BeforeAndAfter,
            colon_numeric_exception: false,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // When exception is disabled, spaces should be added around all colons
        let text = "time := 12:34:56;";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "time := 12 : 34 : 56;");
    }

    #[test]
    fn test_colon_mixed_numeric_and_non_numeric() {
        let options = TextChangeOptions {
            colon: SpaceOperation::BeforeAndAfter,
            colon_numeric_exception: true,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Mix of numeric (no space) and non-numeric (with space) colons
        let text = "var x: Integer; time := 12:34;";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "var x : Integer; time := 12:34;");
    }

    #[test]
    fn test_colon_numeric_exception_with_assignment() {
        let options = TextChangeOptions {
            assign: SpaceOperation::BeforeAndAfter,
            colon: SpaceOperation::BeforeAndAfter,
            colon_numeric_exception: true,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Ensure ':=' assignment is handled separately from single ':'
        let text = "time:=12:34; x:Integer;";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "time := 12:34; x : Integer;");
    }

    #[test]
    fn test_colon_numeric_exception_edge_cases() {
        let options = TextChangeOptions {
            colon: SpaceOperation::BeforeAndAfter,
            colon_numeric_exception: true,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Test edge cases: colon at start, end, and with non-digits
        let text = ":start x:y 3:z end: 12:34";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), ": start x : y 3 : z end : 12:34");
    }

    #[test]
    fn test_colon_numeric_exception_only_after_operation() {
        let options = TextChangeOptions {
            colon: SpaceOperation::After,
            colon_numeric_exception: true,
            trim_trailing_whitespace: false,
            ..Default::default()
        };
        // Test with only 'After' spacing - numeric exception should still work
        let text = "x:Integer; time := 12:34;";
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(result.unwrap(), "x: Integer; time := 12:34;");
    }

    #[test]
    fn test_generic_angle_brackets_no_spacing() {
        let source = "unit Test;\ninterface\nconst\n  AStructures: TEnumerable < TStructure >;\nimplementation\nend.";
        let options = TextChangeOptions::default();
        let (_, context) = crate::parser::parse_with_spacing_context(source).unwrap();
        let result = apply_text_transformation_with_context(
            0,
            source.len(),
            source,
            &options,
            Some(&context),
        );
        assert_eq!(
            result.unwrap().text,
            "unit Test;\ninterface\nconst\n  AStructures: TEnumerable<TStructure>;\nimplementation\nend."
        );
    }

    #[test]
    fn test_unary_minus_no_spacing() {
        let source = "unit Test;\ninterface\nconst\n  A = - 1;\nimplementation\nbegin\n  A := - 1;\n  A := - Foo;\n  A := -Foo(1);\nend.";
        let options = TextChangeOptions::default();
        let (_, context) = crate::parser::parse_with_spacing_context(source).unwrap();
        let result = apply_text_transformation_with_context(
            0,
            source.len(),
            source,
            &options,
            Some(&context),
        );
        assert_eq!(
            result.unwrap().text,
            "unit Test;\ninterface\nconst\n  A = -1;\nimplementation\nbegin\n  A := -1;\n  A := -Foo;\n  A := -Foo(1);\nend."
        );
    }

    #[test]
    fn test_generic_angle_brackets_nested() {
        let source = "unit Test;\ninterface\ntype\n  TMap = TDictionary < String, TList < Integer > >;\n  TNested = TOuter < TInner < Integer > >;\nimplementation\nend.";
        let options = TextChangeOptions::default();
        let (_, context) = crate::parser::parse_with_spacing_context(source).unwrap();
        let result = apply_text_transformation_with_context(
            0,
            source.len(),
            source,
            &options,
            Some(&context),
        );
        assert_eq!(
            result.unwrap().text,
            "unit Test;\ninterface\ntype\n  TMap = TDictionary<String, TList<Integer>>;\n  TNested = TOuter<TInner<Integer>>;\nimplementation\nend."
        );
    }

    #[test]
    fn test_unary_plus_spacing() {
        let source = "unit Test;\ninterface\nconst\n  A = + 1;\nimplementation\nbegin\n  A := + Foo;\n  A := +Foo(1);\n  A := + (1 + 2);\nend.";
        let options = TextChangeOptions::default();
        let (_, context) = crate::parser::parse_with_spacing_context(source).unwrap();
        let result = apply_text_transformation_with_context(
            0,
            source.len(),
            source,
            &options,
            Some(&context),
        );
        assert_eq!(
            result.unwrap().text,
            "unit Test;\ninterface\nconst\n  A = +1;\nimplementation\nbegin\n  A := +Foo;\n  A := +Foo(1);\n  A := +(1 + 2);\nend."
        );
    }

    #[test]
    fn test_exponent_sign_no_spacing() {
        let source = "unit Test;\ninterface\nconst\n  A=1E-12;\n  B=1E+12;\nimplementation\nend.";
        let options = TextChangeOptions::default();
        let (_, context) = crate::parser::parse_with_spacing_context(source).unwrap();
        let result = apply_text_transformation_with_context(
            0,
            source.len(),
            source,
            &options,
            Some(&context),
        );
        assert_eq!(
            result.unwrap().text,
            "unit Test;\ninterface\nconst\n  A = 1E-12;\n  B = 1E+12;\nimplementation\nend."
        );
    }

    #[test]
    fn test_preserve_indentation_for_multiline_expression_operators() {
        let text = "begin\n  X :=\n      A\n    -  B\n    +  C;\nend.";
        let options = TextChangeOptions::default();
        let result = apply_text_changes(text, &options, 0, None);
        assert_eq!(
            result.unwrap(),
            "begin\n  X :=\n      A\n    - B\n    + C;\nend."
        );
    }
}
