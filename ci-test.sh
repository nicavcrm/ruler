#!/bin/bash

# CI/CD Test Script for Ruler
# Designed for continuous integration environments

set -e

# Exit codes
EXIT_SUCCESS=0
EXIT_COMPILATION_FAILED=1
EXIT_TESTS_FAILED=2
EXIT_INTEGRATION_FAILED=3

# Logging
log_info() {
    echo "[INFO] $1"
}

log_error() {
    echo "[ERROR] $1" >&2
}

log_success() {
    echo "[SUCCESS] $1"
}

# Test compilation
test_compilation() {
    log_info "Testing compilation..."

    if ! cargo check; then
        log_error "Compilation check failed"
        exit $EXIT_COMPILATION_FAILED
    fi

    if ! cargo build; then
        log_error "Build failed"
        exit $EXIT_COMPILATION_FAILED
    fi

    log_success "Compilation passed"
}

# Run unit tests
run_unit_tests() {
    log_info "Running unit tests..."

    if ! cargo test; then
        log_error "Unit tests failed"
        exit $EXIT_TESTS_FAILED
    fi

    log_success "Unit tests passed"
}

# Basic integration test
test_integration() {
    log_info "Running integration tests..."

    # Setup
    local test_dir="ci_test"
    rm -rf "$test_dir"
    mkdir -p "$test_dir"/{input,output}

    # Create test files
    cat > "$test_dir/input/test1.mdc" << 'EOF'
---
description: "CI Test 1"
globs: ["*.ts"]
alwaysApply: false
---
Test content 1
EOF

    cat > "$test_dir/input/test2.mdc" << 'EOF'
---
description: "CI Test 2"
globs: "*.js,*.jsx"
alwaysApply: true
---
Test content 2
EOF

    # Test c2g
    if ! cargo run -- c2g --from "$test_dir/input" --to "$test_dir/output"; then
        log_error "c2g conversion failed"
        rm -rf "$test_dir"
        exit $EXIT_INTEGRATION_FAILED
    fi

    # Test g2c
    if ! cargo run -- g2c --from "$test_dir/output" --to "$test_dir/reverse"; then
        log_error "g2c conversion failed"
        rm -rf "$test_dir"
        exit $EXIT_INTEGRATION_FAILED
    fi

    # Validate files exist
    local expected_files=(
        "$test_dir/output/test1.instructions.md"
        "$test_dir/output/test2.instructions.md"
        "$test_dir/reverse/test1.mdc"
        "$test_dir/reverse/test2.mdc"
    )

    for file in "${expected_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Expected file not found: $file"
            rm -rf "$test_dir"
            exit $EXIT_INTEGRATION_FAILED
        fi
    done

    # Cleanup
    rm -rf "$test_dir"
    log_success "Integration tests passed"
}

# Main execution
main() {
    log_info "Starting CI tests for Ruler"

    test_compilation
    run_unit_tests
    test_integration

    log_success "All CI tests passed successfully"
    exit $EXIT_SUCCESS
}

# Parse command line arguments
VERBOSE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  -v, --verbose    Enable verbose output"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Set verbose mode
if [ "$VERBOSE" = true ]; then
    set -x
fi

# Run main function
main
