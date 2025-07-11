#!/bin/bash

# Quick Test Script for Ruler
# A simplified version for rapid testing during development

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Quick compilation and unit test
echo "🔨 Building and testing..."

# Compile
if cargo check --quiet; then
    print_status "Compilation successful"
else
    print_error "Compilation failed"
    exit 1
fi

# Unit tests
if cargo test --quiet; then
    print_status "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Quick integration test
echo -e "\n🧪 Running quick integration test..."

# Setup
TEST_DIR="quick_test"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR/input"
mkdir -p "$TEST_DIR/output"

# Copy test files from fixtures
cp fixtures/cursor/single-string.mdc "$TEST_DIR/input/test.mdc"
cp fixtures/cursor/empty-metadata.mdc "$TEST_DIR/input/empty-test.mdc"

# Test conversion c2g
if cargo run --quiet -- c2g --from "$TEST_DIR/input" --to "$TEST_DIR/output" > /dev/null 2>&1; then
    print_status "c2g conversion works"
else
    print_error "c2g conversion failed"
    exit 1
fi

# Check output
if [ -f "$TEST_DIR/output/test.instructions.md" ]; then
    print_status "Output file created correctly"

    # Check content (test.mdc is single-string fixture with *.js)
    if grep -q "applyTo.*js" "$TEST_DIR/output/test.instructions.md"; then
        print_status "Content converted correctly"
    else
        print_error "Content conversion failed"
        exit 1
    fi
else
    print_error "Output file not found"
    exit 1
fi

# Check empty metadata output
if [ -f "$TEST_DIR/output/empty-test.instructions.md" ]; then
    if grep -A2 "^---" "$TEST_DIR/output/empty-test.instructions.md" | grep -q "^description:$" && \
       grep -A2 "^---" "$TEST_DIR/output/empty-test.instructions.md" | grep -q "^applyTo:$"; then
        print_status "Empty metadata fields preserved correctly"
    else
        print_error "Empty metadata preservation failed"
        exit 1
    fi
else
    print_error "Empty metadata test output file not found"
    exit 1
fi

# Test reverse conversion
if cargo run --quiet -- g2c --from "$TEST_DIR/output" --to "$TEST_DIR/reverse" > /dev/null 2>&1; then
    print_status "g2c conversion works"
else
    print_error "g2c conversion failed"
    exit 1
fi

# Cleanup
rm -rf "$TEST_DIR"

echo -e "\n${GREEN}🎉 All quick tests passed!${NC}"
echo "Use './test.sh' for comprehensive testing."
