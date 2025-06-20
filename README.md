# Ruler: Cursor and GitHub Copilot Rules Converter

**Ruler** is a command-line tool for seamless, bidirectional conversion between [Cursor](https://cursor.sh/)'s `.mdc` rule files and [GitHub Copilot](https://copilot.github.com/)'s `.md` instruction files. It simplifies managing AI-assisted coding rules by allowing developers to maintain a single source of truth and convert them as needed.

This tool is designed for developers who use both Cursor and GitHub Copilot and want to synchronize their custom instructions and rules across both platforms.

## Features

- **Bidirectional Conversion**: Convert rules from Cursor to GitHub Copilot (`c2g`) and back (`g2c`).
- **File & Directory Mapping**: Automatically handles file extension changes (`.mdc` ↔ `.md`) and directory structures (`.cursor/rules` ↔ `.github/instructions`).
- **YAML Frontmatter Transformation**: Intelligently converts metadata between Cursor's and GitHub Copilot's YAML frontmatter schemas.
- **Content Preservation**: Keeps your rule content in Markdown untouched during conversion.
- **Nested Structure Support**: Preserves nested directory structures within the rules folders.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/ruler.git
   cd ruler
   ```

2. Build the release binary:
   ```bash
   cargo build --release
   ```

3. The executable will be available at `target/release/ruler`.

## Usage

The basic command structure is:

```bash
ruler [COMMAND] [OPTIONS]
```

### Commands

- `c2g`: Convert from Cursor (`.mdc`) to GitHub Copilot (`.instructions.md`).
- `g2c`: Convert from GitHub Copilot (`.instructions.md`) to Cursor (`.mdc`).

### Arguments

Both source and target directories are now optional and have sensible defaults:

- **For `c2g` (Cursor to GitHub Copilot)**:
  - Default source: `.cursor/rules`
  - Default target: `.github/instructions`
- **For `g2c` (GitHub Copilot to Cursor)**:
  - Default source: `.github/instructions`
  - Default target: `.cursor/rules`

### Options

- `-f, --from <FOLDER>`: Override the default source directory.
- `-t, --to <FOLDER>`: Override the default target directory.
- `-h, --help`: Print help information.
- `-V, --version`: Print version information.

### Examples

- **Convert Cursor rules to GitHub Copilot instructions** (using defaults):
  ```bash
  ruler c2g
  ```

- **Convert GitHub Copilot instructions to Cursor rules** (using defaults):
  ```bash
  ruler g2c
  ```

- **Convert with custom directories**:
  ```bash
  ruler c2g --from custom/rules --to custom/instructions
  ruler g2c --from .github/instructions --to .cursor/rules
  ```

- **Legacy syntax** (still supported):
  ```bash
  ruler c2g --from .cursor/rules --to .github/instructions
  ruler g2c --from .github/instructions --to .cursor/rules
  ```

## Format Conversion Specifications

### File and Directory Structure

- **Cursor to GitHub Copilot (`c2g`)**:
  - Default input: `.mdc` files from `.cursor/rules/`
  - Default output: `.instructions.md` files in `.github/instructions/`
- **GitHub Copilot to Cursor (`g2c`)**:
  - Default input: `.instructions.md` files from `.github/instructions/`
  - Default output: `.mdc` files in `.cursor/rules/`

### YAML Frontmatter Field Mapping

| Cursor (`.mdc`) | GitHub Copilot (`.md`) | Conversion Logic |
| :--- | :--- | :--- |
| `description` | `description` | Direct 1:1 mapping. |
| `globs` (array) | `applyTo` (string) | `c2g`: Joins the array into a comma-separated string.<br>`g2c`: Splits the comma-separated string into an array. |
| `alwaysApply` (bool) | `applyTo` (string) | `c2g`: If `true`, sets `applyTo` to `"**"`.<br>`g2c`: If `applyTo` is `"**"`, sets `alwaysApply` to `true`. |

## Flexible Configuration

### Globs Field Format

The tool now supports both string and array formats for the `globs` field in Cursor `.mdc` files:

```yaml
# Array format (recommended)
globs: ["*.ts", "*.tsx"]

# String format (also supported)
globs: "*.ts"
```

Both formats will be converted correctly to GitHub Copilot's `applyTo` field format.

## Sample File Examples

### Cursor Rule (`.cursor/rules/typescript.mdc`)

```yaml
---
description: "Enforce TypeScript best practices"
globs: ["**/src/*.ts", "**/src/*.tsx"]
alwaysApply: false
---

Always use `const` or `let` instead of `var`.
```

### GitHub Copilot Instruction (`.github/instructions/typescript.instructions.md`)

This is the output after running `ruler c2g`.

```yaml
---
description: "Enforce TypeScript best practices"
applyTo: "**/src/*.ts,**/src/*.tsx"
---

Always use `const` or `let` instead of `var`.
```

## For Developers

### Build Instructions

To build the project for development (with debug symbols):

```bash
cargo build
```

The executable will be at `target/debug/ruler`.

### Running Tests

To run the test suite:

```bash
cargo test
```

## Edge Cases and Limitations

- **Unsupported Cursor Rules**: Cursor's `Agent Requested` and `Manual` rule types do not have a direct equivalent in GitHub Copilot. While the content of these rules will be converted, they will not be automatically triggered in GitHub Copilot. You will need to reference them manually.
- **Primary Instruction File**: GitHub Copilot has a special `.github/copilot-instructions.md` file for rules that are always active. A Cursor rule with `alwaysApply: true` is a good candidate for this file. The tool currently converts it to a standard instruction with `applyTo: "**"`, but you can move the content to the primary instruction file manually.

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/your-username/ruler/issues).

## License

This project is licensed under the MIT License
