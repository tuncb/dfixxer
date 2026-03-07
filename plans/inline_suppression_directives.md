# Inline Suppression Directives Spec

## Purpose

Add file-local directives that disable and later re-enable `dfixxer` processing for selected regions of a Pascal source file.

This feature must suppress:
- AST-driven structural replacements
- zero-length insertion replacements
- whole-file lexical text transformations

The behavior must be deterministic, byte-range based, and independent of parser recovery quality.

## Goals

- Let users protect small source regions without creating a separate config file.
- Keep syntax valid Pascal in all supported source files.
- Make suppression work consistently across the full transformation pipeline.
- Avoid accidental activation from prose comments, strings, or compiler directives.
- Preserve disabled regions byte-for-byte.

## Non-Goals

- Per-transform suppression such as "disable only uses sorting".
- Nested suppression scopes.
- Raw `# dfixxer off` syntax.
- Compiler-directive syntax such as `{$DFIXXER OFF}`.
- Same-line inline directives attached to code.

## Canonical Syntax

Document and prefer:

```pascal
// dfixxer:off
...
// dfixxer:on
```

Accepted equivalent aliases:

```pascal
{ dfixxer:off }
...
{ dfixxer:on }
```

```pascal
(* dfixxer:off *)
...
(* dfixxer:on *)
```

## Recognition Rules

A comment is recognized as a `dfixxer` directive only if all of the following are true:

1. The token is a normal Pascal comment:
   - `// ...`
   - `{ ... }`
   - `(* ... *)`
2. The token is not a compiler directive / preprocessor token:
   - `{$...}` is never a `dfixxer` directive
3. The directive comment is single-line:
   - `// ...` is naturally single-line
   - `{ ... }` and `(* ... *)` must open and close on the same logical line
4. The directive occupies an otherwise empty logical line:
   - allowed: optional leading whitespace, one directive comment, optional trailing whitespace
   - not allowed: code before the comment
   - not allowed: code after the comment
   - not allowed: multiple comments on the same line
5. The normalized comment body, after removing wrappers and trimming surrounding horizontal whitespace, matches exactly one of:
   - `dfixxer:off`
   - `dfixxer:on`
6. Matching is case-insensitive for ASCII letters.

Examples that must be recognized:

```pascal
// dfixxer:off
//DFIXXER:ON
{dfixxer:off}
(*   dfixxer:on   *)
```

Examples that must not be recognized:

```pascal
# dfixxer off
{$IFDEF DEBUG}
x := 1; // dfixxer:off
{ dfixxer:off } x := 1;
// please keep dfixxer:off in mind
(* dfixxer:off
*)
```

## Range Semantics

Formatting state is enabled at the start of every file.

Recognized directives control a set of half-open suppressed ranges:

- `off` starts suppression after the directive line ending
- `on` ends suppression at the start byte of the directive line
- a suppressed range is represented as `[start, end)`

Consequences:

- the directive line itself is not part of the suppressed payload range
- content between the directives is protected
- content after the `on` line is enabled again

If the `off` directive is the last line in the file and there is no following line ending, suppression begins at the end of the directive token.

If an `off` directive is not matched by a later `on`, suppression continues to EOF.

If `off` is immediately followed by `on`, the suppressed range may be empty. This is valid and should produce no edits inside the empty range.

## Directive Line Preservation

Recognized directive comments must be preserved verbatim:

- do not reformat spacing inside the directive comment
- do not rewrite casing inside the directive comment
- do not trim or normalize the directive line beyond unchanged file copying

This avoids churn such as converting `{dfixxer:off}` to `{ dfixxer:off }`.

## State Machine

Per file, directives are processed in source order with a simple two-state machine:

- `Enabled`
- `Disabled`

Transitions:

- `Enabled` + `off` -> `Disabled`
- `Disabled` + `on` -> `Enabled`

Invalid transitions:

- `Enabled` + `on`
- `Disabled` + `off`

Nested suppression is not supported. Invalid transitions do not abort the file; they are ignored with a warning.

## Warning Rules

Warnings should include at least the file path and line number.

Warn and continue for:

- `on` encountered while already enabled
- `off` encountered while already disabled
- EOF reached while still disabled
- an exact directive body found in an unsupported placement:
  - same line as code
  - multiline block comment
  - non-standalone comment line

Do not warn for:

- normal comments that merely mention `dfixxer`
- strings containing directive-like text
- compiler directives

## Pipeline Semantics

Suppression must apply to every transformation path in the current pipeline.

### 1. Structural AST Replacements

This includes:

- uses section formatting
- unit / program header formatting
- single-keyword section formatting

Rule:

- if the final replacement span overlaps any suppressed range, skip the entire replacement

Overlap for non-empty replacements:

- replacement overlaps suppression when
  - `replacement.start < suppressed.end`
  - and `replacement.end > suppressed.start`

### 2. Zero-Length Insertions

This includes:

- procedure / function `()` insertion
- inherited-call expansion insertion

Rule:

- if the insertion point lies inside a suppressed range, skip the insertion

Containment for insertions:

- insertion is suppressed when
  - `suppressed.start <= point`
  - and `point < suppressed.end`

### 3. Adjusted Replacement Boundaries

Some structural transforms can expand their effective edit range using line-based helpers. Suppression checks must run against the final replacement span after any such adjustment, not only against the original AST node span.

### 4. Whole-File Lexical Text Transformations

The lexical text pass currently formats the gaps between structural replacements. It must additionally exclude:

- suppressed ranges
- recognized directive comment lines
- existing parser error ranges already treated as preserve zones

The text transformation pass must operate only on enabled, non-suppressed, non-error segments.

### 5. Merge / Apply Phase

Replacement validation should reject or skip overlapping edits that would cross a suppressed boundary. Suppression should be enforced before final merge, not only during candidate generation.

## Boundary Rules

These rules define behavior at suppression edges:

- code on the directive line is never allowed, so there is no same-line ambiguity
- the first byte after an `off` line is suppressed
- the first byte of an `on` line is not suppressed
- an insertion exactly at `suppressed.end` is allowed
- an insertion exactly at `suppressed.start` is blocked

## Interaction With Existing Behavior

### Comments Inside Uses Sections

`uses` transforms already skip sections containing comment or preprocessor nodes at the same level as unit names. This feature does not change that rule.

Directive comments are intended to wrap an entire section, not be inserted inside the comma-separated unit list.

### Parser Errors

Suppression discovery must not depend on the AST. The directive scan should be lexical over source text or comment tokens so that suppression still works in files with parser recovery drift.

### Line Endings

Suppression boundaries must use actual source byte positions and handle:

- LF
- CRLF
- CR

The feature must not normalize line endings by itself.

### Strings

Text inside string literals must never act as a directive.

### Directive Comments Themselves

Directive comments are control markers, not payload text. Once recognized, they should be passed through unchanged even when surrounding regions are enabled.

## Recommended Implementation Shape

Add a dedicated lexical directive collector, for example:

- `src/suppression.rs`

Recommended responsibilities:

- scan source text for normal comment tokens
- classify recognized directives
- compute normalized suppressed ranges
- compute directive-line exclusion ranges
- expose helper predicates:
  - `overlaps_range(start, end)`
  - `contains_point(pos)`
  - `subtract_from_sections(...)`

Recommended data shape:

```rust
pub struct SuppressionContext {
    pub suppressed_ranges: Vec<(usize, usize)>,
    pub directive_ranges: Vec<(usize, usize)>,
    pub warnings: Vec<SuppressionWarning>,
}
```

All ranges should be normalized and merged where possible.

## Test Matrix

### Unit Tests

Add focused tests for:

- recognizes `// dfixxer:off` and `// dfixxer:on`
- recognizes `{ ... }` and `(* ... *)` aliases
- case-insensitive matching
- ignores raw `# dfixxer off`
- ignores `{$...}` directives
- ignores directive text inside strings
- warns on `on` without prior `off`
- warns on repeated `off`
- warns on unsupported inline placement
- unmatched `off` suppresses until EOF
- CRLF and LF boundary calculations
- zero-length insertion blocked inside suppressed range
- insertion at suppression end allowed
- final adjusted structural replacement span blocked when overlapping suppression
- directive comment text preserved verbatim

### End-to-End Fixtures

Add `test-data/update/` fixtures for:

- disabled uses section remains unchanged while later uses section is formatted
- disabled lexical spacing region remains unchanged while later region is normalized
- disabled procedure declaration does not get `()`
- disabled bare `inherited;` does not expand
- re-enable resumes formatting after protected block
- brace-comment and paren-star directive aliases
- unsupported inline directive placement is ignored and warned

## Acceptance Criteria

The feature is complete when:

1. A user can wrap a region with `dfixxer:off` / `dfixxer:on` comment directives.
2. No `dfixxer` transformation changes bytes inside the protected region.
3. Code before and after the protected region continues to format normally.
4. Recognized directive comments remain unchanged.
5. Invalid directive sequences warn but do not fail the file.
6. Behavior is covered by unit tests and end-to-end fixtures.
