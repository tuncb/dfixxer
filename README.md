# dfixxer — Delphi code formatter (uses-section)

Version: 0.0.1

A small command-line tool that reformats and sorts Delphi/Pascal uses sections in-place, powered by tree-sitter.

- Parses Pascal via tree-sitter-pascal
- Formats the uses section with a chosen style and indentation
- Sorts unit names, with optional namespace prioritization
- Optionally rewrites short unit names into fully-qualified ones
- Leaves sections untouched when grammar errors or unsupported constructs are detected

## Install / build

Requires Rust (stable).

- Build debug: `cargo build`
- Build release: `cargo build --release`
- Run tests: `cargo test`

The binary is `dfixxer` (on Windows: `dfixxer.exe`).

## Usage

Commands:

- update <filename> [--config <path>]
  - Reformats and sorts the uses section(s) in the given Pascal file, modifying it in-place.
  - If `--config` isn’t provided, the tool looks for `dfixxer.toml` starting from the file’s folder and walking up parent folders. If none is found, built-in defaults are used.
- init-config <filename>
  - Writes a default configuration file to the given path.
  - No additional flags are accepted for this command.

Exit status: non-zero on error; prints a message to stderr.

Notes during update:
- If a uses section or its parent has a parse error, it is skipped and a warning is printed.
- If a uses section contains preprocessor (`pp`) or comment nodes at the same level, it’s treated as unsupported and skipped with a warning.

### Examples (PowerShell)

- Update a file using discovered or default config

```
./target/debug/dfixxer update .\examples\simple.pas
```

- Update a file with an explicit config

```
./target/debug/dfixxer update .\examples\simple.pas --config .\dfixxer.toml
```

- Create a default config file

```
./target/debug/dfixxer init-config .\dfixxer.toml
```

## Configuration (dfixxer.toml)

The config file is TOML. All keys are optional; unspecified keys use defaults.

Fields (source: `src/options.rs`):

- indentation: string
  - Indentation used for uses section lines.
  - Default: two spaces: "  "
- uses_section_style: enum
  - One of: "CommaAtTheBeginning" | "CommaAtTheEnd"
  - Controls whether the comma appears at the start or end of each line.
  - Default: "CommaAtTheEnd"
- override_sorting_order: array of strings
  - A list of namespace prefixes to prioritize. Units starting with any of these prefixes (followed by a dot) are sorted first among themselves, then the rest are sorted normally.
  - Default: []
- modules_names_to_update: array of strings
  - Each entry is "Prefix:Name". When the tool encounters the short unit `Name`, it is rewritten to `Prefix.Name` before sorting/formatting.
  - Default: []

### Example dfixxer.toml

```
indentation = "    "            # 4 spaces
uses_section_style = "CommaAtTheEnd"
override_sorting_order = ["System", "Vcl"]
modules_names_to_update = [
  "System:Classes",     # Classes -> System.Classes
  "Vcl:Dialogs"         # Dialogs -> Vcl.Dialogs
]
```

## What gets changed

Given a uses section like:

```
uses UnitC, UnitA, Classes;
```

With the example config above, it becomes:

```
uses
    System.Classes,
    UnitA,
    UnitC;
```

Style "CommaAtTheBeginning" produces:

```
uses
    System.Classes
    , UnitA
    , UnitC
    ;
```

## Limitations

- Sections with parse errors are skipped (a warning is printed).
- Sections containing preprocessor directives or comments at the same level as unit names are skipped (unsupported).
- Only the uses section is reformatted; other code is not modified.

## License

TBD.
