#!/bin/bash

# Ruler Testing Script
# Automates comprehensive testing of the ruler tool including unit tests and integration tests

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test directories
TEST_DIR="test_temp"
CURSOR_DIR="$TEST_DIR/.cursor/rules"
GITHUB_DIR="$TEST_DIR/.github/instructions"
OUTPUT_DIR="$TEST_DIR/output"
FIXTURES_DIR="fixtures"

# Helper functions
print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Cleanup function
cleanup() {
    if [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
        print_success "Cleaned up test directory"
    fi
}

# Setup test environment
setup_test_env() {
    print_header "Setting up test environment"

    # Cleanup any existing test directory
    cleanup

    # Create test directories
    mkdir -p "$CURSOR_DIR"
    mkdir -p "$GITHUB_DIR"
    mkdir -p "$OUTPUT_DIR"

    print_success "Test directories created"
}

# Create test files with various formats
create_test_files() {
    print_header "Setting up test files from fixtures"

    # Copy all cursor fixture files to test directory
    if [ -d "$FIXTURES_DIR/cursor" ]; then
        cp -r "$FIXTURES_DIR/cursor"/* "$CURSOR_DIR/"
        print_success "Cursor test files copied from fixtures"
    else
        print_error "Cursor fixtures directory not found: $FIXTURES_DIR/cursor"
        exit 1
    fi

    # Verify key test files exist
    local required_files=(
        "$CURSOR_DIR/standard-array.mdc"
        "$CURSOR_DIR/single-string.mdc"
        "$CURSOR_DIR/comma-separated.mdc"
        "$CURSOR_DIR/always-apply.mdc"
        "$CURSOR_DIR/empty-metadata.mdc"
        "$CURSOR_DIR/no-frontmatter.mdc"
        "$CURSOR_DIR/extension-conversion.mdc"
        "$CURSOR_DIR/nested/deep/nested-rule.mdc"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            print_error "Required test file missing: $file"
            exit 1
        fi
    done

    print_success "All test files verified"
}

# Create reverse test files (GitHub format)
create_github_test_files() {
    print_header "Setting up GitHub instruction test files from fixtures"

    # Copy all GitHub fixture files to test directory
    if [ -d "$FIXTURES_DIR/github" ]; then
        cp -r "$FIXTURES_DIR/github"/* "$GITHUB_DIR/"
        print_success "GitHub instruction files copied from fixtures"
    else
        print_error "GitHub fixtures directory not found: $FIXTURES_DIR/github"
        exit 1
    fi

    # Verify key test files exist
    local required_files=(
        "$GITHUB_DIR/reverse-test.instructions.md"
        "$GITHUB_DIR/universal.instructions.md"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            print_error "Required GitHub test file missing: $file"
            exit 1
        fi
    done

    print_success "All GitHub test files verified"
}

# Run unit tests
run_unit_tests() {
    print_header "Running unit tests"

    if cargo test --quiet; then
        print_success "All unit tests passed"
    else
        print_error "Unit tests failed"
        exit 1
    fi
}

# Test compilation
test_compilation() {
    print_header "Testing compilation"

    if cargo check --quiet; then
        print_success "Code compiles successfully"
    else
        print_error "Compilation failed"
        exit 1
    fi

    if cargo build --quiet; then
        print_success "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Test Cursor to GitHub conversion
test_c2g_conversion() {
    print_header "Testing Cursor to GitHub conversion (c2g)"

    # Test with default directories
    if cargo run --quiet -- c2g --from "$CURSOR_DIR" --to "$OUTPUT_DIR/github"; then
        print_success "c2g conversion completed"
    else
        print_error "c2g conversion failed"
        exit 1
    fi

    # Verify output files exist
    local expected_files=(
        "$OUTPUT_DIR/github/standard-array.instructions.md"
        "$OUTPUT_DIR/github/single-string.instructions.md"
        "$OUTPUT_DIR/github/comma-separated.instructions.md"
        "$OUTPUT_DIR/github/always-apply.instructions.md"
        "$OUTPUT_DIR/github/empty-metadata.instructions.md"
        "$OUTPUT_DIR/github/no-frontmatter.instructions.md"
        "$OUTPUT_DIR/github/extension-conversion.instructions.md"
        "$OUTPUT_DIR/github/nested/deep/nested-rule.instructions.md"
    )

    for file in "${expected_files[@]}"; do
        if [ -f "$file" ]; then
            print_success "Created: $(basename "$file")"
        else
            print_error "Missing: $file"
            exit 1
        fi
    done
}

# Test GitHub to Cursor conversion
test_g2c_conversion() {
    print_header "Testing GitHub to Cursor conversion (g2c)"

    if cargo run --quiet -- g2c --from "$GITHUB_DIR" --to "$OUTPUT_DIR/cursor"; then
        print_success "g2c conversion completed"
    else
        print_error "g2c conversion failed"
        exit 1
    fi

    # Verify output files exist
    local expected_files=(
        "$OUTPUT_DIR/cursor/reverse-test.mdc"
        "$OUTPUT_DIR/cursor/universal.mdc"
    )

    for file in "${expected_files[@]}"; do
        if [ -f "$file" ]; then
            print_success "Created: $(basename "$file")"
        else
            print_error "Missing: $file"
            exit 1
        fi
    done
}

# Test round-trip conversion
test_round_trip() {
    print_header "Testing round-trip conversion"

    # Convert c2g then g2c
    cargo run --quiet -- c2g --from "$CURSOR_DIR" --to "$OUTPUT_DIR/round1" || exit 1
    cargo run --quiet -- g2c --from "$OUTPUT_DIR/round1" --to "$OUTPUT_DIR/round2" || exit 1

    print_success "Round-trip conversion completed"

    # Verify some key conversions
    if [ -f "$OUTPUT_DIR/round2/comma-separated.mdc" ]; then
        # Check that comma-separated globs were properly handled
        if grep -q "globs:" "$OUTPUT_DIR/round2/comma-separated.mdc"; then
            print_success "Comma-separated globs properly converted in round-trip"
        else
            print_error "Comma-separated globs conversion failed in round-trip"
        fi
    fi
}

# Test default directory behavior
test_default_directories() {
    print_header "Testing default directory behavior"

    # Setup default directories in test environment
    local default_test_dir="$TEST_DIR/default_test"
    mkdir -p "$default_test_dir/.cursor/rules"
    mkdir -p "$default_test_dir/.github/instructions"

    # Copy a test file to default location
    cp "$FIXTURES_DIR/cursor/standard-array.mdc" "$default_test_dir/.cursor/rules/"

    # Test c2g with defaults (run from test directory)
    local original_dir=$(pwd)
    cd "$default_test_dir"

    if cargo run --quiet --manifest-path "$original_dir/Cargo.toml" -- c2g; then
        print_success "c2g with default directories works"

        if [ -f ".github/instructions/standard-array.instructions.md" ]; then
            print_success "Default output file created correctly"
        else
            print_error "Default output file not found"
        fi
    else
        print_error "c2g with default directories failed"
    fi

    cd "$original_dir"
}

# Validate file contents
validate_conversions() {
    print_header "Validating conversion contents"

    # Check comma-separated conversion
    if [ -f "$OUTPUT_DIR/github/comma-separated.instructions.md" ]; then
        if grep -q "applyTo.*optimization.*integration" "$OUTPUT_DIR/github/comma-separated.instructions.md"; then
            print_success "Comma-separated globs correctly converted to applyTo"
        else
            print_error "Comma-separated globs conversion validation failed"
        fi
    fi

    # Check always apply conversion
    if [ -f "$OUTPUT_DIR/github/always-apply.instructions.md" ]; then
        if grep -q "applyTo.*\*\*" "$OUTPUT_DIR/github/always-apply.instructions.md"; then
            print_success "alwaysApply correctly converted to applyTo: **"
        else
            print_error "alwaysApply conversion validation failed"
        fi
    fi

    # Check empty metadata conversion
    if [ -f "$OUTPUT_DIR/github/empty-metadata.instructions.md" ]; then
        if grep -A2 "^---" "$OUTPUT_DIR/github/empty-metadata.instructions.md" | grep -q "^description:$" && \
           grep -A2 "^---" "$OUTPUT_DIR/github/empty-metadata.instructions.md" | grep -q "^applyTo:$"; then
            print_success "Empty metadata fields correctly preserved in output"
        else
            print_error "Empty metadata fields conversion validation failed"
            print_warning "Expected 'description:' and 'applyTo:' with no values"
        fi
    fi

    # Check extension conversion (.mdc to .instructions.md)
    if [ -f "$OUTPUT_DIR/github/extension-conversion.instructions.md" ]; then
        if grep -q "applyTo.*\[name\]\.instructions\.md" "$OUTPUT_DIR/github/extension-conversion.instructions.md"; then
            print_success ".mdc extensions correctly converted to .instructions.md"
        else
            print_error "Extension conversion validation failed"
            print_warning "Expected '[name].instructions.md' in applyTo field"
        fi
    fi

    # Check reverse conversion
    if [ -f "$OUTPUT_DIR/cursor/universal.mdc" ]; then
        if grep -q "alwaysApply: true" "$OUTPUT_DIR/cursor/universal.mdc"; then
            print_success "Universal applyTo correctly converted to alwaysApply: true"
        else
            print_error "Universal applyTo conversion validation failed"
        fi
    fi
}

# Test error handling
test_error_handling() {
    print_header "Testing error handling"

    # Test with non-existent directory
    if ! cargo run --quiet -- c2g --from "non-existent-dir" --to "$OUTPUT_DIR/error-test" 2>/dev/null; then
        print_success "Properly handles non-existent source directory"
    else
        print_warning "Should fail with non-existent directory, but didn't"
    fi

    # Test with invalid YAML (use malformed fixture)
    if ! cargo run --quiet -- c2g --from "$FIXTURES_DIR/malformed" --to "$OUTPUT_DIR/error-output" 2>/dev/null; then
        print_success "Properly handles malformed YAML"
    else
        print_warning "Should fail with malformed YAML, but didn't"
    fi
}

# Test help and version commands
test_cli_commands() {
    print_header "Testing CLI commands"

    if cargo run --quiet -- --help > /dev/null; then
        print_success "Help command works"
    else
        print_error "Help command failed"
    fi

    if cargo run --quiet -- --version > /dev/null; then
        print_success "Version command works"
    else
        print_error "Version command failed"
    fi
}

# Performance test
performance_test() {
    print_header "Running performance test"

    # Create multiple test files
    mkdir -p "$OUTPUT_DIR/perf-test"
    for i in {1..50}; do
        cat > "$OUTPUT_DIR/perf-test/rule-$i.mdc" << EOF
---
description: "Performance test rule $i"
globs: ["*.test$i.ts", "**/*spec$i.js"]
alwaysApply: false
---

# Performance Test Rule $i

This is a performance test rule.
EOF
    done

    # Time the conversion
    local start_time=$(date +%s.%N)
    cargo run --quiet -- c2g --from "$OUTPUT_DIR/perf-test" --to "$OUTPUT_DIR/perf-output" > /dev/null
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)

    print_success "Performance test completed in ${duration}s (50 files)"
}

# Main execution
main() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════╗"
    echo "║        Ruler Testing Suite           ║"
    echo "║                                      ║"
    echo "║  Comprehensive testing of the        ║"
    echo "║  Cursor <-> GitHub Copilot           ║"
    echo "║  rules conversion tool               ║"
    echo "╚══════════════════════════════════════╝"
    echo -e "${NC}\n"

    # Trap cleanup on exit
    trap cleanup EXIT

    # Run all tests
    test_compilation
    run_unit_tests
    setup_test_env
    create_test_files
    create_github_test_files
    test_c2g_conversion
    test_g2c_conversion
    test_round_trip
    test_default_directories
    validate_conversions
    test_error_handling
    test_cli_commands
    performance_test

    print_header "Test Summary"
    print_success "All tests completed successfully!"
    echo -e "\n${GREEN}🎉 Ruler tool is working correctly!${NC}\n"
}

# Check if bc is available for performance test
if ! command -v bc &> /dev/null; then
    print_warning "bc command not found, performance timing will be skipped"
    performance_test() {
        print_header "Skipping performance test (bc not available)"
    }
fi

# Run main function
main "$@"
