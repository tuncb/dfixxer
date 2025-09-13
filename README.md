# dfixxer


Version: 0.1.0

A command-line tool that reformats Delphi/Pascal files.

## Install / build

Requires Rust (stable).

- Build debug: `cargo build`
- Build release: `cargo build --release`
- Run tests: `cargo test`

The binary is `dfixxer` (on Windows: `dfixxer.exe`).

## Usage

### Command Syntax

```
dfixxer [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

### Global Options

- `-l, --log-level <LEVEL>`: Set the logging level
  - Possible values: `off`, `error`, `warn`, `info`, `debug`, `trace`
  - Default: No logging output

### Commands

#### `update` - Reformat file in-place

```
dfixxer update <filename> [--config <path>]
```

Reformats and sorts the uses section(s) in the given Pascal file, modifying it in-place.

**Arguments:**
- `<filename>`: Path to the Pascal file to update (required)

**Options:**
- `--config <path>`: Path to configuration file
  - If not provided, searches for `dfixxer.toml` starting from the file's directory and walking up parent directories
  - If no config file is found, uses built-in defaults

#### `check` - Preview changes without modifying

```
dfixxer check <filename> [--config <path>]
```

Shows what changes would be made to the uses section(s) without modifying the file.

**Arguments:**
- `<filename>`: Path to the Pascal file to check (required)

**Options:**
- `--config <path>`: Path to configuration file (same behavior as `update`)

**Exit Code:**
- Returns the number of replacements that would be made as the exit code
- `0` if no changes are needed
- `N` (where N > 0) if N replacements would be made
- `1` if an error occurred (with error message printed to stderr)

#### `init-config` - Create default configuration

```
dfixxer init-config <filename>
```

Creates a default configuration file at the specified path.

**Arguments:**
- `<filename>`: Path where the configuration file should be created (required)

#### `parse` - Debug: Show AST

```
dfixxer parse <filename>
```

Parses a Pascal file and prints its Abstract Syntax Tree (AST) for debugging purposes.

**Arguments:**
- `<filename>`: Path to the Pascal file to parse (required)

#### `parse-debug` - Debug: Show detailed parsing information

```
dfixxer parse-debug <filename>
```

Parses a Pascal file and prints detailed debug information including parser output for troubleshooting.

**Arguments:**
- `<filename>`: Path to the Pascal file to parse with debug output (required)

### Exit Codes

- `0`: Success (no changes needed for `check` command, or successful completion for other commands)
- `N` (where N > 0): For `check` command only - indicates N replacements would be made
- `1`: Error occurred (message printed to stderr)

### Processing Notes

- If a uses section or its parent has a parse error, it is skipped and a warning is printed
- If a uses section contains preprocessor directives (`{$...}`) or comment nodes at the same level as unit names, it's treated as unsupported and skipped with a warning

### Examples

#### Update a file using auto-discovered or default config

```pwsh
./target/debug/dfixxer update .\examples\simple.pas
```

#### Update a file with explicit config and debug logging

```pwsh
./target/debug/dfixxer --log-level debug update .\examples\simple.pas --config .\dfixxer.toml
```

#### Check what changes would be made without modifying the file

```pwsh
./target/debug/dfixxer check .\examples\simple.pas
```

#### Check for changes and use exit code in scripts

```pwsh
# Check file and capture the number of replacements
./target/debug/dfixxer check .\examples\simple.pas
$replacements = $LASTEXITCODE
if ($replacements -eq 0) {
    Write-Host "File is already properly formatted"
} elseif ($replacements -gt 0) {
    Write-Host "File needs $replacements changes"
} else {
    Write-Host "Error checking file"
}
```

#### Create a default config file

```pwsh
./target/debug/dfixxer init-config .\dfixxer.toml
```

#### Get help for any command

```pwsh
./target/debug/dfixxer --help
./target/debug/dfixxer update --help
```

## Configuration (dfixxer.toml)

The configuration file uses TOML format. All keys are optional; unspecified keys use built-in defaults.

### Configuration Options

#### `indentation` (string)
- **Purpose**: Indentation used for uses section lines
- **Default**: `"  "` (two spaces)
- **Examples**:
  - `"    "` (four spaces)
  - `"\t"` (tab character)

#### `uses_section_style` (enum)
- **Purpose**: Controls comma placement in formatted uses sections
- **Values**:
  - `"CommaAtTheEnd"` - Comma appears at the end of each line (default)
  - `"CommaAtTheBeginning"` - Comma appears at the start of each line
- **Default**: `"CommaAtTheEnd"`

#### `line_ending` (enum)
- **Purpose**: Controls line ending style in output
- **Values**:
  - `"Auto"` - Use platform default (CRLF on Windows, LF elsewhere) (default)
  - `"Crlf"` - Force Windows-style line endings (\r\n)
  - `"Lf"` - Force Unix-style line endings (\n)
- **Default**: `"Auto"`

#### `override_sorting_order` (array of strings)
- **Purpose**: Namespace prefixes to prioritize during sorting
- **Behavior**: Units starting with these prefixes (followed by a dot) are sorted first among themselves, then remaining units are sorted alphabetically
- **Default**: `[]` (empty array)
- **Example**: `["System", "Vcl", "FireDAC"]`

#### `module_names_to_update` (array of strings)
- **Purpose**: Map short unit names to fully-qualified names
- **Format**: Each entry is `"Prefix:ShortName"`
- **Behavior**: When the tool encounters `ShortName`, it rewrites it to `Prefix.ShortName` before sorting/formatting
- **Default**: Extensive list of 258 built-in mappings for System, Winapi, and other common namespaces
- **Example**: `["System:Classes", "Vcl:Dialogs", "FireDAC:Comp.Client"]`

#### `transformations` (object)
- **Purpose**: Controls which transformation features are enabled
- **Default**: All transformations enabled
- **Properties**:
  - `enable_uses_section` (boolean) - Enable uses section formatting (default: `true`)
  - `enable_unit_program_section` (boolean) - Enable unit/program section processing (default: `true`)
  - `enable_single_keyword_sections` (boolean) - Enable single keyword section processing (default: `true`)
  - `enable_procedure_section` (boolean) - Enable procedure section processing (default: `true`)

### Complete Example Configuration

```toml
# Use 4-space indentation
indentation = "    "

# Put commas at the beginning of lines
uses_section_style = "CommaAtTheBeginning"

# Force Unix-style line endings
line_ending = "Lf"

# Prioritize System and Vcl namespaces
override_sorting_order = ["System", "Vcl", "FireDAC"]

# Automatically qualify common unit names
module_names_to_update = [
    "System:Classes",
    "System:SysUtils",
    "System:Variants",
    "Vcl:Forms",
    "Vcl:Controls",
    "Vcl:Dialogs",
    "FireDAC:Comp.Client",
    "FireDAC:Stan.Def"
]

# Control which transformations are enabled
[transformations]
enable_uses_section = true
enable_unit_program_section = true
enable_single_keyword_sections = true
enable_procedure_section = true
```

### Configuration File Discovery

When `--config` is not specified:
1. Starts from the target file's directory
2. Looks for `dfixxer.toml` in current directory
3. If not found, walks up parent directories
4. Uses the first `dfixxer.toml` file found
5. If no config file is found, uses built-in defaults

## Formatting Examples

### Input Code
```pascal
uses UnitC, UnitA, Classes, Forms;
```

### With `CommaAtTheEnd` style (default)
```pascal
uses
    System.Classes,
    UnitA,
    UnitC,
    Vcl.Forms;
```

### With `CommaAtTheBeginning` style
```pascal
uses
    System.Classes
    , UnitA
    , UnitC
    , Vcl.Forms
    ;
```

### Configuration Used for Above Examples
```toml
indentation = "    "
override_sorting_order = ["System", "Vcl"]
module_names_to_update = [
    "System:Classes",
    "Vcl:Forms"
]
```

## Limitations

- **Parse errors**: Sections with syntax errors are skipped (warning printed)
- **Unsupported constructs**: Sections containing preprocessor directives (`{$...}`) or comments at the same level as unit names are skipped (warning printed)

## License

Licensed under the Apache License, Version 2.0. See the [LICENSE](license.txt) file for the full license text.

## Contributing

This project is open source. Contributions are welcome through pull requests and issue reports.
