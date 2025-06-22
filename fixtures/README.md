# Test Fixtures

This directory contains test files used by the test scripts (`test.sh` and `quick-test.sh`). These fixtures provide consistent test data for validating the ruler tool's conversion functionality.

## Directory Structure

```
fixtures/
├── cursor/                   # Cursor .mdc test files
│   ├── standard-array.mdc    # Tests standard array format for globs
│   ├── single-string.mdc     # Tests single string format for globs
│   ├── comma-separated.mdc   # Tests comma-separated string format
│   ├── always-apply.mdc      # Tests alwaysApply: true conversion
│   ├── empty-metadata.mdc    # Tests empty metadata field preservation
│   ├── no-frontmatter.mdc    # Tests files without frontmatter
│   └── nested/deep/          # Tests nested directory structure
│       └── nested-rule.mdc
├── github/                   # GitHub Copilot .instructions.md test files
│   ├── reverse-test.instructions.md  # Tests g2c conversion
│   └── universal.instructions.md     # Tests universal apply (applyTo: "**")
└── malformed/                # Files with invalid YAML for error testing
    └── bad.mdc               # Malformed YAML for error handling tests
```

## Test Coverage

### Cursor Format Tests (`cursor/`)

- **standard-array.mdc**: Tests the standard YAML array format for globs: `["*.ts", "*.tsx"]`
- **single-string.mdc**: Tests single string format: `"*.js"`
- **comma-separated.mdc**: Tests comma-separated format: `"glob1,glob2,glob3"`
- **always-apply.mdc**: Tests the `alwaysApply: true` field conversion to `applyTo: "**"`
- **empty-metadata.mdc**: Tests preservation of empty metadata field structure
- **no-frontmatter.mdc**: Tests files without any YAML frontmatter
- **nested/deep/nested-rule.mdc**: Tests nested directory structure preservation

### GitHub Format Tests (`github/`)

- **reverse-test.instructions.md**: Tests conversion from GitHub Copilot back to Cursor format
- **universal.instructions.md**: Tests `applyTo: "**"` conversion to `alwaysApply: true`

### Error Handling Tests (`malformed/`)

- **bad.mdc**: Contains intentionally malformed YAML to test error handling

## Usage

The test scripts automatically copy these fixtures to temporary test directories during execution. This approach provides several benefits:

1. **Consistency**: All tests use the same, version-controlled test data
2. **Maintainability**: Test data is separated from test logic
3. **Reusability**: Fixtures can be used by multiple test scripts
4. **Debugging**: Easy to examine test inputs when debugging failures

## Modifying Fixtures

When adding new test cases:

1. Add the fixture file to the appropriate subdirectory
2. Update the test scripts to include validation for the new case
3. Update this README to document the new fixture

The fixtures should remain minimal and focused on specific test scenarios to keep tests fast and maintainable.
