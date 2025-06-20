# Makefile for Ruler project

.PHONY: help build test quick-test ci-test clean check fmt lint run-c2g run-g2c install release

# Default target
help:
	@echo "Ruler - Cursor <-> GitHub Copilot Rules Converter"
	@echo ""
	@echo "Available targets:"
	@echo "  build       - Build the project"
	@echo "  test        - Run comprehensive tests"
	@echo "  quick-test  - Run quick integration tests"
	@echo "  ci-test     - Run CI/CD tests"
	@echo "  check       - Check code compilation"
	@echo "  fmt         - Format code"
	@echo "  lint        - Run clippy linter"
	@echo "  clean       - Clean build artifacts"
	@echo "  run-c2g     - Run c2g conversion with default paths"
	@echo "  run-g2c     - Run g2c conversion with default paths"
	@echo "  install     - Install the binary"
	@echo "  release     - Build release version"

# Build the project
build:
	cargo build

# Build release version
release:
	cargo build --release

# Check compilation
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy
lint:
	cargo clippy -- -D warnings

# Run unit tests only
unit-test:
	cargo test

# Run comprehensive test suite
test:
	@echo "Running comprehensive test suite..."
	./test.sh

# Run quick tests
quick-test:
	@echo "Running quick tests..."
	./quick-test.sh

# Run CI tests
ci-test:
	@echo "Running CI tests..."
	./ci-test.sh

# Clean build artifacts
clean:
	cargo clean
	rm -rf test_temp quick_test ci_test

# Run c2g conversion (Cursor to GitHub)
run-c2g:
	cargo run -- c2g

# Run g2c conversion (GitHub to Cursor)
run-g2c:
	cargo run -- g2c

# Install the binary
install:
	cargo install --path .

# Development workflow - run this before committing
dev-check: fmt lint check unit-test quick-test
	@echo "✅ All development checks passed!"

# CI workflow
ci: ci-test
	@echo "✅ CI checks passed!"

# Show help when no target is specified
.DEFAULT_GOAL := help
