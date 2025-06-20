# Ruler: Cursor and GitHub Copilot Rules Converter

**Ruler** is a command-line tool for seamless, bidirectional conversion between [Cursor](https://cursor.sh/)'s `.mdc` rule files and [GitHub Copilot](https://copilot.github.com/)'s `.md` instruction files. It simplifies managing AI-assisted coding rules by allowing developers to maintain a single source of truth and convert them as needed.

This tool is designed for developers who use both Cursor and GitHub Copilot and want to synchronize their custom instructions and rules across both platforms.

## Features

- **Bidirectional Conversion**: Convert rules from Cursor to GitHub Copilot (`c2g`) and back (`g2c`).
- **File & Directory Mapping**: Automatically handles file extension changes (`.mdc` ↔ `.md`) and directory structures (`.cursor/rules` ↔ `.github/instructions`).
- **YAML Frontmatter Transformation**: Intelligently converts metadata between Cursor's and GitHub Copilot's YAML frontmatter schemas with support for multiple `globs` formats.
- **Flexible Parsing**: Handles various YAML formats including arrays, strings, comma-separated values, and non-standard formats.
- **Error Resilience**: Continues processing files even if some fail to parse, reporting errors without aborting the entire conversion.
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
| `globs` (array/string) | `applyTo` (string) | `c2g`: Joins arrays or parses comma-separated strings into a comma-separated format.<br>`g2c`: Splits the comma-separated string into an array. Supports multiple input formats. |
| `alwaysApply` (bool) | `applyTo` (string) | `c2g`: If `true`, sets `applyTo` to `"**"`.<br>`g2c`: If `applyTo` is `"**"`, sets `alwaysApply` to `true`. |

## Flexible Configuration

### Globs Field Format

The tool supports multiple formats for the `globs` field in Cursor `.mdc` files, providing maximum compatibility:

```yaml
# Array format (recommended)
globs: ["*.ts", "*.tsx"]

# Single string format
globs: "*.ts"

# Comma-separated string format
globs: "*.ts,*.tsx,**/*.spec.ts"

# Multiple quoted strings format (YAML flow sequence style)
globs: "*.ts", "*.tsx", "**/*.spec.ts"
```

All formats will be converted correctly to GitHub Copilot's `applyTo` field format, and the tool can handle mixed formats within the same project.

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

### Alternative Cursor Rule Formats

The tool also handles these equivalent formats:

```yaml
---
description: "Enforce TypeScript best practices"
globs: "**/src/*.ts", "**/src/*.tsx"  # Multiple quoted strings
alwaysApply: false
---
```

```yaml
---
description: "Enforce TypeScript best practices"
globs: "**/src/*.ts,**/src/*.tsx"  # Comma-separated string
alwaysApply: false
---
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

### Testing

The project includes comprehensive testing scripts to ensure reliability:

#### Quick Testing
For rapid development cycles:
```bash
./quick-test.sh
# or
make quick-test
```

#### Comprehensive Testing
For thorough validation:
```bash
./test.sh
# or
make test
```

#### CI/CD Testing
For continuous integration environments:
```bash
./ci-test.sh
# or
make ci-test
```

#### Unit Tests Only
```bash
cargo test
# or
make unit-test
```

#### Development Workflow
Before committing code:
```bash
make dev-check  # Runs fmt, lint, check, unit-test, and quick-test
```

### Test Coverage

The test suite covers:
- **Unit tests** for core functionality
- **Integration tests** with various file formats and `globs` configurations
- **Round-trip conversions** (Cursor → GitHub → Cursor)
- **Error handling** for malformed files and non-standard YAML formats
- **Multiple `globs` formats** (arrays, strings, comma-separated, quoted strings)
- **Default directory behavior**
- **Performance testing** with multiple files
- **CLI command validation**

### Code Quality

Use these commands to maintain code quality:

```bash
# Format code
cargo fmt
# or
make fmt

# Run linter
cargo clippy
# or
make lint

# Check compilation
cargo check
# or
make check
```

## Edge Cases and Limitations

- **Unsupported Cursor Rules**: Cursor's `Agent Requested` and `Manual` rule types do not have a direct equivalent in GitHub Copilot. While the content of these rules will be converted, they will not be automatically triggered in GitHub Copilot. You will need to reference them manually.
- **Primary Instruction File**: GitHub Copilot has a special `.github/copilot-instructions.md` file for rules that are always active. A Cursor rule with `alwaysApply: true` is a good candidate for this file. The tool currently converts it to a standard instruction with `applyTo: "**"`, but you can move the content to the primary instruction file manually.
- **YAML Format Compatibility**: The tool handles non-standard YAML formats (like `globs: "pattern1", "pattern2"`) by preprocessing them into valid YAML before parsing. This ensures maximum compatibility with existing rule files.
- **Error Handling**: If individual files fail to parse, the tool reports the error and continues processing other files rather than aborting the entire conversion.

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/your-username/ruler/issues).

## License

This project is licensed under the MIT License
