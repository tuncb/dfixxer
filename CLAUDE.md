# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dfixxer is a Rust command-line tool that reformats and sorts Delphi/Pascal `uses` sections in-place using tree-sitter-pascal for parsing. The tool provides three main commands: `update` (modify files), `check` (preview changes), and `init-config` (create default configuration).

Debug output can be shown by `parse` (see tree-sitter output) and `parse-debug` (see parser output)

## Build and Development Commands

- **Build debug**: `cargo build`
- **Build release**: `cargo build --release`
- **Run tests**: `cargo test`
- **Run with logging**: `cargo run -- --log-level debug <command> <args>`
- **See github issue**: `gh issue view <id> --repo tuncb/dfixxer`

The binary is `dfixxer` (or `dfixxer.exe` on Windows) and can be found in `target/debug/` or `target/release/`.

## Code style

- Prefer free functions instead of member ones.

## Architecture

### Core Modules

- `main.rs`: CLI entry point, timing collection, and orchestration
- `arguments.rs`: Command-line argument parsing using clap
- `options.rs`: Configuration file handling (dfixxer.toml) with TOML parsing
- `parser.rs`: Tree-sitter-pascal integration for AST parsing
- `replacements.rs`: Text replacement engine for applying changes to source files
- `transform_XXX`: Each transformation rule is implemented in a separate file.
- `dfixxer_error.rs`: Error handling types

### Execution flow
- see process_file function in `main.rs` for the check and update workflow

### Processing Flow

1. Parse command-line arguments
2. Discover or load configuration file (walks up directory tree)
3. Parse Pascal file using tree-sitter
4. Extract and validate uses sections
5. Apply sorting, formatting, and namespace transformations
6. Generate text replacements
7. Apply changes (for `update`) or display preview (for `check`)

## Testing and Examples

### Examples
- Add new examples to examples/ folder.
- The parse and parse-debug commands to the app can be used to examine the tree-sitter and parser.rs output

### Testing
- Add small tests to the source code itself
- there is e2e testing (test_update_smoke), add update tests to test-data/update folder. we recursively go through and compare XXX.original.test.pas files to XXX.correct.test.pas

## Development Notes

- Uses Rust edition 2024
- Depends on tree-sitter-pascal for parsing
- Implements comprehensive logging with env_logger
- Includes timing collection for performance monitoring
- Supports configuration file discovery by walking up parent directories