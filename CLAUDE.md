# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dfixxer is a Rust command-line tool that reformats and sorts Delphi/Pascal `uses` sections in-place using tree-sitter-pascal for parsing. The tool provides three main commands: `update` (modify files), `check` (preview changes), and `init-config` (create default configuration).

## Build and Development Commands

- **Build debug**: `cargo build`
- **Build release**: `cargo build --release`
- **Run tests**: `cargo test`
- **Run with logging**: `cargo run -- --log-level debug <command> <args>`

The binary is `dfixxer` (or `dfixxer.exe` on Windows) and can be found in `target/debug/` or `target/release/`.

## Architecture

### Core Modules

- `main.rs`: CLI entry point, timing collection, and orchestration
- `arguments.rs`: Command-line argument parsing using clap
- `options.rs`: Configuration file handling (dfixxer.toml) with TOML parsing
- `parser.rs`: Tree-sitter-pascal integration for AST parsing
- `uses_section.rs`: Core logic for transforming uses sections (sorting, formatting, namespace handling)
- `replacements.rs`: Text replacement engine for applying changes to source files
- `dfixxer_error.rs`: Error handling types

### Key Configuration Options

The tool uses `dfixxer.toml` configuration files with these main settings:
- `indentation`: String for indentation (default: two spaces)
- `uses_section_style`: "CommaAtTheEnd" or "CommaAtTheBeginning"
- `line_ending`: "Auto", "Crlf", or "Lf"
- `override_sorting_order`: Array of namespace prefixes to prioritize
- `module_names_to_update`: Array of "Prefix:ShortName" mappings for unit qualification

### Processing Flow

1. Parse command-line arguments
2. Discover or load configuration file (walks up directory tree)
3. Parse Pascal file using tree-sitter
4. Extract and validate uses sections
5. Apply sorting, formatting, and namespace transformations
6. Generate text replacements
7. Apply changes (for `update`) or display preview (for `check`)

## Testing and Examples

The project includes example Pascal files in an `examples/` directory for testing. The tool handles parse errors gracefully by skipping problematic sections and printing warnings.

## Development Notes

- Uses Rust edition 2024
- Depends on tree-sitter-pascal for parsing
- Implements comprehensive logging with env_logger
- Includes timing collection for performance monitoring
- Supports configuration file discovery by walking up parent directories