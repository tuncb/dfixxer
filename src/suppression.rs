#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuppressionWarningKind {
    UnsupportedPlacement,
    UnmatchedOn,
    RepeatedOff,
    UnterminatedOff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressionWarning {
    pub line: usize,
    pub kind: SuppressionWarningKind,
}

impl SuppressionWarning {
    pub fn message(&self) -> String {
        match self.kind {
            SuppressionWarningKind::UnsupportedPlacement => {
                "inline dfixxer directive must be a standalone single-line comment".to_string()
            }
            SuppressionWarningKind::UnmatchedOn => {
                "encountered 'dfixxer:on' while formatting is already enabled".to_string()
            }
            SuppressionWarningKind::RepeatedOff => {
                "encountered 'dfixxer:off' while formatting is already disabled".to_string()
            }
            SuppressionWarningKind::UnterminatedOff => {
                "encountered 'dfixxer:off' without a later matching 'dfixxer:on'".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SuppressionContext {
    pub suppressed_ranges: Vec<(usize, usize)>,
    pub directive_ranges: Vec<(usize, usize)>,
    pub warnings: Vec<SuppressionWarning>,
}

impl SuppressionContext {
    pub fn text_exclusion_ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = self.directive_ranges.clone();
        ranges.extend_from_slice(&self.suppressed_ranges);
        normalize_ranges(&mut ranges);
        ranges
    }

    pub fn suppresses_replacement(&self, start: usize, end: usize) -> bool {
        if start == end {
            contains_point(&self.suppressed_ranges, start)
        } else {
            overlaps_range(&self.suppressed_ranges, start, end)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DirectiveKind {
    Off,
    On,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CommentKind {
    Line,
    Brace,
    ParenStar,
}

struct CommentToken<'a> {
    kind: CommentKind,
    text: &'a str,
    start: usize,
    end: usize,
    is_single_line: bool,
}

pub fn collect_suppression_context(source: &str) -> SuppressionContext {
    let line_starts = build_line_starts(source);
    let mut context = SuppressionContext::default();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut disabled_start: Option<usize> = None;
    let mut disabled_origin_line: Option<usize> = None;

    while i < bytes.len() {
        match bytes[i] {
            b'\'' => {
                i = consume_string(bytes, i);
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                let start = i;
                i += 2;
                while i < bytes.len() && bytes[i] != b'\r' && bytes[i] != b'\n' {
                    i += 1;
                }
                handle_comment(
                    source,
                    &line_starts,
                    &mut context,
                    &mut disabled_start,
                    &mut disabled_origin_line,
                    CommentToken {
                        kind: CommentKind::Line,
                        text: &source[start..i],
                        start,
                        end: i,
                        is_single_line: true,
                    },
                );
            }
            b'{' => {
                let start = i;
                i += 1;
                if i < bytes.len() && bytes[i] == b'$' {
                    while i < bytes.len() && bytes[i] != b'}' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                    continue;
                }

                let mut is_single_line = true;
                while i < bytes.len() && bytes[i] != b'}' {
                    if bytes[i] == b'\r' || bytes[i] == b'\n' {
                        is_single_line = false;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                handle_comment(
                    source,
                    &line_starts,
                    &mut context,
                    &mut disabled_start,
                    &mut disabled_origin_line,
                    CommentToken {
                        kind: CommentKind::Brace,
                        text: &source[start..i],
                        start,
                        end: i,
                        is_single_line,
                    },
                );
            }
            b'(' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                let start = i;
                i += 2;
                let mut is_single_line = true;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b')') {
                    if bytes[i] == b'\r' || bytes[i] == b'\n' {
                        is_single_line = false;
                    }
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2;
                } else {
                    i = bytes.len();
                }
                handle_comment(
                    source,
                    &line_starts,
                    &mut context,
                    &mut disabled_start,
                    &mut disabled_origin_line,
                    CommentToken {
                        kind: CommentKind::ParenStar,
                        text: &source[start..i],
                        start,
                        end: i,
                        is_single_line,
                    },
                );
            }
            _ => {
                i += 1;
            }
        }
    }

    if let Some(start) = disabled_start
        && start < source.len()
    {
        context.suppressed_ranges.push((start, source.len()));
    }
    if let Some(line) = disabled_origin_line {
        context.warnings.push(SuppressionWarning {
            line,
            kind: SuppressionWarningKind::UnterminatedOff,
        });
    }

    normalize_ranges(&mut context.suppressed_ranges);
    normalize_ranges(&mut context.directive_ranges);
    context
}

pub fn overlaps_range(ranges: &[(usize, usize)], start: usize, end: usize) -> bool {
    for &(range_start, range_end) in ranges {
        if range_start >= end {
            break;
        }
        if range_end <= start {
            continue;
        }
        return true;
    }
    false
}

pub fn contains_point(ranges: &[(usize, usize)], point: usize) -> bool {
    for &(range_start, range_end) in ranges {
        if range_start > point {
            break;
        }
        if range_start <= point && point < range_end {
            return true;
        }
    }
    false
}

fn handle_comment(
    source: &str,
    line_starts: &[usize],
    context: &mut SuppressionContext,
    disabled_start: &mut Option<usize>,
    disabled_origin_line: &mut Option<usize>,
    token: CommentToken<'_>,
) {
    let directive_kind = comment_directive_kind(token.kind, token.text);
    let warning_kind = comment_directive_kind_with_trimmed_whitespace(token.kind, token.text);
    if directive_kind.is_none() && warning_kind.is_none() {
        return;
    }

    let line_start = find_line_start(source, token.start);
    let line_end_without_newline = find_line_end_without_newline(source, token.end);
    let line_end_with_newline = find_line_end_with_newline(source, line_end_without_newline);
    let standalone_line = is_standalone_comment_line(
        source,
        line_start,
        token.start,
        token.end,
        line_end_without_newline,
    );
    let line = line_number_at(line_starts, token.start);

    let Some(recognized_kind) = directive_kind else {
        context.warnings.push(SuppressionWarning {
            line,
            kind: SuppressionWarningKind::UnsupportedPlacement,
        });
        return;
    };

    if !token.is_single_line || !standalone_line {
        context.warnings.push(SuppressionWarning {
            line,
            kind: SuppressionWarningKind::UnsupportedPlacement,
        });
        return;
    }

    context
        .directive_ranges
        .push((line_start, line_end_with_newline));

    if let Some(start) = *disabled_start
        && start < line_start
    {
        context.suppressed_ranges.push((start, line_start));
    }

    match (*disabled_start, recognized_kind) {
        (None, DirectiveKind::Off) => {
            *disabled_start = Some(line_end_with_newline);
            *disabled_origin_line = Some(line);
        }
        (None, DirectiveKind::On) => {
            context.warnings.push(SuppressionWarning {
                line,
                kind: SuppressionWarningKind::UnmatchedOn,
            });
        }
        (Some(_), DirectiveKind::On) => {
            *disabled_start = None;
            *disabled_origin_line = None;
        }
        (Some(_), DirectiveKind::Off) => {
            context.warnings.push(SuppressionWarning {
                line,
                kind: SuppressionWarningKind::RepeatedOff,
            });
            *disabled_start = Some(line_end_with_newline);
        }
    }
}

fn consume_string(bytes: &[u8], mut index: usize) -> usize {
    index += 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                    index += 2;
                } else {
                    return index + 1;
                }
            }
            b'\r' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\n' {
                    return index + 2;
                }
                return index + 1;
            }
            b'\n' => {
                return index + 1;
            }
            _ => {
                index += 1;
            }
        }
    }
    index
}

fn comment_directive_kind(kind: CommentKind, text: &str) -> Option<DirectiveKind> {
    let body = trim_horizontal(comment_body(kind, text));
    parse_directive_kind(body)
}

fn comment_directive_kind_with_trimmed_whitespace(
    kind: CommentKind,
    text: &str,
) -> Option<DirectiveKind> {
    let body = comment_body(kind, text).trim();
    parse_directive_kind(body)
}

fn comment_body(kind: CommentKind, text: &str) -> &str {
    match kind {
        CommentKind::Line => &text[2..],
        CommentKind::Brace => &text[1..text.len().saturating_sub(1)],
        CommentKind::ParenStar => &text[2..text.len().saturating_sub(2)],
    }
}

fn parse_directive_kind(text: &str) -> Option<DirectiveKind> {
    if text.eq_ignore_ascii_case("dfixxer:off") {
        Some(DirectiveKind::Off)
    } else if text.eq_ignore_ascii_case("dfixxer:on") {
        Some(DirectiveKind::On)
    } else {
        None
    }
}

fn trim_horizontal(text: &str) -> &str {
    text.trim_matches(|c| c == ' ' || c == '\t')
}

fn find_line_start(source: &str, position: usize) -> usize {
    if position == 0 {
        return 0;
    }

    let bytes = source.as_bytes();
    for i in (0..position).rev() {
        if bytes[i] == b'\n' || bytes[i] == b'\r' {
            return i + 1;
        }
    }
    0
}

fn find_line_end_without_newline(source: &str, position: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = position;
    while i < bytes.len() && bytes[i] != b'\n' && bytes[i] != b'\r' {
        i += 1;
    }
    i
}

fn find_line_end_with_newline(source: &str, position: usize) -> usize {
    let bytes = source.as_bytes();
    if position >= bytes.len() {
        return position;
    }
    if bytes[position] == b'\r' {
        if position + 1 < bytes.len() && bytes[position + 1] == b'\n' {
            return position + 2;
        }
        return position + 1;
    }
    if bytes[position] == b'\n' {
        return position + 1;
    }
    position
}

fn is_standalone_comment_line(
    source: &str,
    line_start: usize,
    token_start: usize,
    token_end: usize,
    line_end_without_newline: usize,
) -> bool {
    let mut prefix = &source[line_start..token_start];
    if line_start == 0
        && let Some(stripped) = prefix.strip_prefix('\u{feff}')
    {
        prefix = stripped;
    }
    is_horizontal_whitespace(prefix)
        && is_horizontal_whitespace(&source[token_end..line_end_without_newline])
}

fn is_horizontal_whitespace(text: &str) -> bool {
    text.chars().all(|c| c == ' ' || c == '\t')
}

fn build_line_starts(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut line_starts = vec![0usize];
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    line_starts.push(i + 2);
                    i += 2;
                } else {
                    line_starts.push(i + 1);
                    i += 1;
                }
            }
            b'\n' => {
                line_starts.push(i + 1);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    line_starts
}

fn line_number_at(line_starts: &[usize], position: usize) -> usize {
    match line_starts.binary_search(&position) {
        Ok(index) => index + 1,
        Err(index) => index,
    }
}

fn normalize_ranges(ranges: &mut Vec<(usize, usize)>) {
    ranges.retain(|(start, end)| start < end);
    if ranges.is_empty() {
        return;
    }

    ranges.sort_unstable_by_key(|(start, end)| (*start, *end));

    let mut merged = Vec::with_capacity(ranges.len());
    let mut current = ranges[0];

    for &(start, end) in ranges.iter().skip(1) {
        if start <= current.1 {
            current.1 = current.1.max(end);
        } else {
            merged.push(current);
            current = (start, end);
        }
    }

    merged.push(current);
    *ranges = merged;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_line_comment_directives() {
        let source = "x := 1;\n// dfixxer:off\n  y:=1+2;\n// dfixxer:on\nz:=3+4;\n";
        let context = collect_suppression_context(source);

        assert_eq!(
            context.suppressed_ranges,
            vec![(
                source.find("  y").unwrap(),
                source.find("// dfixxer:on").unwrap()
            )]
        );
        assert_eq!(context.directive_ranges.len(), 2);
        assert!(context.warnings.is_empty());
    }

    #[test]
    fn test_collect_alias_directives() {
        let source =
            "{ dfixxer:off }\na:=1;\n{ dfixxer:on }\n(* dfixxer:off *)\nb:=2;\n(* dfixxer:on *)\n";
        let context = collect_suppression_context(source);

        assert_eq!(context.suppressed_ranges.len(), 2);
        assert!(context.directive_ranges.len() >= 3);
        assert!(context.warnings.is_empty());
    }

    #[test]
    fn test_ignores_hash_syntax_and_strings() {
        let source = "# dfixxer off\nmsg := '// dfixxer:off';\n";
        let context = collect_suppression_context(source);

        assert!(context.suppressed_ranges.is_empty());
        assert!(context.directive_ranges.is_empty());
        assert!(context.warnings.is_empty());
    }

    #[test]
    fn test_warns_for_inline_directive_placement() {
        let source = "x := 1; // dfixxer:off\n";
        let context = collect_suppression_context(source);

        assert!(context.suppressed_ranges.is_empty());
        assert_eq!(
            context.warnings,
            vec![SuppressionWarning {
                line: 1,
                kind: SuppressionWarningKind::UnsupportedPlacement
            }]
        );
    }

    #[test]
    fn test_warns_for_repeated_off_and_splits_suppression_around_directive_line() {
        let source = "// dfixxer:off\na:=1;\n// dfixxer:off\nb:=2;\n// dfixxer:on\n";
        let context = collect_suppression_context(source);
        let second_off = source
            .match_indices("// dfixxer:off")
            .nth(1)
            .map(|(index, _)| index)
            .unwrap();

        assert_eq!(
            context.suppressed_ranges,
            vec![
                (source.find("a:=1;").unwrap(), second_off),
                (
                    source.find("b:=2;").unwrap(),
                    source.find("// dfixxer:on").unwrap()
                ),
            ]
        );
        assert_eq!(
            context.warnings,
            vec![SuppressionWarning {
                line: 3,
                kind: SuppressionWarningKind::RepeatedOff
            }]
        );
    }

    #[test]
    fn test_warns_for_on_without_off() {
        let source = "// dfixxer:on\nx:=1;\n";
        let context = collect_suppression_context(source);

        assert!(context.suppressed_ranges.is_empty());
        assert_eq!(
            context.warnings,
            vec![SuppressionWarning {
                line: 1,
                kind: SuppressionWarningKind::UnmatchedOn
            }]
        );
    }

    #[test]
    fn test_unterminated_off_suppresses_until_eof() {
        let source = "// dfixxer:off\r\na:=1;\r\n";
        let context = collect_suppression_context(source);

        assert_eq!(
            context.suppressed_ranges,
            vec![(source.find("a:=1;").unwrap(), source.len())]
        );
        assert_eq!(
            context.warnings,
            vec![SuppressionWarning {
                line: 1,
                kind: SuppressionWarningKind::UnterminatedOff
            }]
        );
    }

    #[test]
    fn test_multiline_block_comment_with_exact_body_warns() {
        let source = "(*\n dfixxer:off\n*)\na:=1;\n";
        let context = collect_suppression_context(source);

        assert!(context.suppressed_ranges.is_empty());
        assert_eq!(
            context.warnings,
            vec![SuppressionWarning {
                line: 1,
                kind: SuppressionWarningKind::UnsupportedPlacement
            }]
        );
    }

    #[test]
    fn test_suppresses_zero_length_insertions_inside_ranges() {
        let source = "// dfixxer:off\na:=1;\n// dfixxer:on\n";
        let context = collect_suppression_context(source);
        let point = source.find("a:=1;").unwrap() + 1;

        assert!(context.suppresses_replacement(point, point));
    }

    #[test]
    fn test_insertion_at_suppression_end_is_allowed() {
        let source = "// dfixxer:off\na:=1;\n// dfixxer:on\n";
        let context = collect_suppression_context(source);
        let point = source.find("// dfixxer:on").unwrap();

        assert!(!context.suppresses_replacement(point, point));
    }
}
