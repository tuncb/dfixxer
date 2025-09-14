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
Transformation Rules / Invariants

Never introduce duplicate spaces where a required space already exists.
Never inject a leading space at beginning of text for a "Before" rule if there is no preceding character.
Multi-operator parsing must not consume unrelated characters.
String/comment detection must prevent interior spacing changes but still allow trimming of line endings (if enabled).
Identity replacements remain None if formatting produces identical text.
Edge Cases Covered by Tests (must preserve)

Escaped quotes inside strings ('' sequence).
Unterminated string before newline: state resets, subsequent code processed.
Mixed comments and code sequences.
Consecutive punctuation (e.g., a,,b) does not force space insertion before second punctuation but may insert after depending on rule.
Numeric colon exception interactions with assignment (:=) and regular colons.
CR, LF, and CRLF line endings handled uniformly for trimming.
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