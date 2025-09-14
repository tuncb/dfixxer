# ARCHITECTURE

Very brief architectural snapshot of the `dfixxer` tool.

## Code Style
- Prefer small, free functions; keep loops / orchestration at higher levels (e.g. in `main.rs`).
- Transform functions operate on a single logical item; composition happens outside.

## Core Modules
- `main.rs` – CLI entry, command dispatch, timing collection.
- `arguments.rs` – Command-line parsing (clap).
- `options.rs` – Config (`dfixxer.toml`) discovery + load (walks parent dirs).
- `parser.rs` – Tree-sitter Pascal parsing + AST access helpers.
- `replacements.rs` – Text replacement engine (accumulate & apply edits).
- `transform_*.rs` – Individual transformation rules (uses section, unit/program, procedure, single-keyword sections, etc.).
- `transformer_utility.rs` – Shared helpers for transformations.
- `dfixxer_error.rs` – Error types and unifying error handling.

## Execution Flow (Check / Update)
1. Parse CLI args.
2. Locate / load configuration (walk upwards for `dfixxer.toml`).
3. Parse Pascal source into AST (tree-sitter-pascal).
4. Identify & validate `uses` and other transform targets.
5. Apply ordered transformation steps (sorting, formatting, namespace adjustments, section rewrites).
6. Accumulate text replacements.
7. Output preview (`check`) or write edits in-place (`update`).
8. (Optional) Debug: `parse` (raw tree-sitter) / `parse-debug` (parser instrumentation).

## Testing Strategy
- Inline small/targeted tests near logic where practical.
- End-to-end update tests in `test-data/update/`: compare `*.original.test.pas` → `*.correct.test.pas`.
- Smoke / CLI tests in `tests/` (e.g. argument handling & overall command behavior).
- Add new transformation cases by pairing original/correct fixture files; recursive walker asserts diffs.

## Observability & Performance
- Logging controlled by `--log-level`; rich debug via specialized parse commands.
- Timing collection inside `main.rs` to monitor transformation phases.

## Extending
Add a new transformation:
1. Create `transform_<name>.rs` implementing focused logic.
2. Reuse helpers in `transformer_utility.rs`.
3. Register / call it in the orchestration path (follow existing transform module patterns).
4. Add E2E fixtures (original + correct) plus any focused unit tests.
