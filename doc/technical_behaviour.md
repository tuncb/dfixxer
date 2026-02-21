# Technical Behaviour

## Line Ending Selection in Generated Output

### Summary

- Generated/newly introduced line breaks use `options.line_ending`.
- Default is `Auto`, which resolves to `\r\n` on Windows and `\n` on non-Windows targets.
- The tool does not infer line endings from the current file content.

### Decision Path

1. Configuration is loaded for the target file (`src/main.rs`, `process_file`), with support for per-pattern custom configs.
2. `Options.line_ending` defaults to `LineEnding::Auto` (`src/options.rs`).
3. `LineEnding` resolves via `Display`:
   - `Auto` -> platform default (`cfg!(windows)` => `\r\n`, otherwise `\n`)
   - `Crlf` -> `\r\n`
   - `Lf` -> `\n`

### Where It Is Applied

- Uses-section reformatting joins lines using `options.line_ending.to_string()` (`src/transform_uses_section.rs`).
- If a transformed keyword/section must be moved to a new line, the prepended newline is `options.line_ending` (`src/transformer_utility.rs`).

### Where It Is Not Applied

- Final output write is a direct `std::fs::write` of the merged string (`src/replacements.rs`).
- Unchanged source slices are copied as-is during merge, preserving their original line endings (`src/replacements.rs`).
- Text transformation logic handles encountered `\n` and `\r` characters but does not globally normalize entire-file line endings (`src/transform_text.rs`).

### Net Effect

- New/generated line breaks follow config (`line_ending`, default `Auto`).
- Existing untouched content keeps its original line endings.
- Mixed line endings can exist in a file after an update when original content and generated content differ.
