Reusable Prompt (Requirements Specification)
You are to maintain or extend the Rust function: apply_text_transformations(original_source: &str, replacements: Vec<TextReplacement>, options: &TextChangeOptions) -> Vec<TextReplacement>.

Implement or modify it under these strict requirements:

Purpose

Iterate over mutable TextReplacement entries.
Skip any with is_final == true (no mutation of their text).
For replacements with text: Some(String), rewrite the string via apply_text_changes.
For identity replacements (text: None), derive the original text slice using [start..end], run transformations, and set text = Some(modified) only if modified differs from original (avoid unnecessary allocation / diff noise).
Return the updated vector preserving order.
Data Model Assumptions

start and end form a valid UTF-8 slice boundary within original_source.
Non-overlapping or ordering are not enforced here (caller responsibility).
is_final signifies "do not touch" even if spacing options would alter content.
Delegated Formatting (apply_text_changes)

Handles: commas, semicolons, arithmetic (+ - * /), comparison (< > = <= >= <>), assignment variants (:= += -= *= /=), colon typing, and division (/) spacing via SpaceOperation:
Variants: NoChange, Before, After, BeforeAndAfter.
Skips inserting spaces inside:
Pascal/Delphi string literals with escaped quotes ('It''s')
Line comments (// ...)
Brace comments ({ ... })
Paren-star comments ((* ... *))
Trims trailing whitespace per line if trim_trailing_whitespace is true (operates line-buffered).
Colon numeric exception (colon_numeric_exception == true): suppresses spacing around single : when both adjacent chars are digits (e.g., time literals 12:34), but still spaces type annotations (x: Integer).
Multi-character operators recognized atomically so spacing rules apply once (<= >= <> := += etc.).
Bidirectional normalization: not only adds required spaces but also removes any surplus horizontal whitespace (spaces, tabs) immediately before/after handled tokens so that spacing matches the configured policy exactly (0 or 1 space as dictated by the rule and context).
Transformation Rules / Invariants

Never introduce duplicate spaces where a required space already exists; likewise collapse runs of one or more spaces/tabs into exactly one space when a space is required, or zero when no space is required.
Never inject a leading space at beginning of text for a "Before" rule if there is no preceding character.
Multi-operator parsing must not consume unrelated characters.
String/comment detection must prevent interior spacing changes but still allow trimming of line endings (if enabled).
Identity replacements remain None if formatting produces identical text.
When removing surplus spaces, do not alter spacing inside string or comment regions (except for configured end-of-line trimming).
For Before / After / BeforeAndAfter:
	- Before: ensure exactly one space precedes the token (unless at start-of-string or already preceded by another punctuation forming a cluster where internal spacing is intentionally suppressed); remove any spaces after unless an After or BeforeAndAfter rule for that token also applies.
	- After: ensure exactly one space follows the token (unless next character is end-of-line, end-of-input, or a consecutive identical operator character such as the second '+' in '++'); remove any preceding surplus spaces unless a Before rule requires one.
	- BeforeAndAfter: ensure exactly one space on both sides subject to start/end boundaries, colon numeric exception, and consecutive identical operator suppression.
Consecutive identical single-character operators (e.g. '++', '--', '==') are treated as a tight cluster: no space inserted between the identical characters, but cluster itself is spaced according to rules relative to surrounding tokens.
Normalization removes extra spaces that previously existed around handled tokens even if those spaces were manually inserted (idempotency: running the transform again yields identical output).
Edge Cases Covered by Tests (must preserve)

Escaped quotes inside strings ('' sequence).
Unterminated string before newline: state resets, subsequent code processed.
Mixed comments and code sequences.
Consecutive punctuation (e.g., a,,b) does not force space insertion before second punctuation but may insert after depending on rule.
Numeric colon exception interactions with assignment (:=) and regular colons.
CR, LF, and CRLF line endings handled uniformly for trimming.
Normalization must not create or preserve trailing whitespace at line ends when trim_trailing_whitespace is enabled; it also must not add a space that would then be immediately trimmed (avoid churn).
Performance Expectations

Single pass O(n) over characters per transformed string.
Avoid unnecessary allocations (reserve capacity; only allocate replacement text when changed).
No regex usage; rely on char peeking.
Safety / Correctness

Do not panic on empty input or zero-length replacements.
Assume caller validates index bounds; function should not adjust them.
Preserve untouched fields of TextReplacement besides optional text.
Extensibility Guidelines

Adding new token spacing types: extend TextChangeOptions and integrate in apply_text_changes, not in apply_text_transformations.
If future semantic skipping zones (e.g., generics, attributes) are needed, expand the state machine cleanly.
Keep operator handling centralized (consider table-driven approach if variants grow).
Output

Return updated Vec<TextReplacement> with minimal alterations (only where formatting applied).
Final replacements remain bitwise identical (apart from pass-through in vector).
Testing Expectations

Maintain existing test suite behavior (spacing, trimming, exception logic).
Add tests when new operator categories or skip regions are introduced.
Provide any implementation changes in idiomatic Rust, preserving these constraints. Do not remove existing safety or state machine logic. Avoid regressions in comment/string detection.