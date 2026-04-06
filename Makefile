.PHONY: all build check fmt lint test clean

# Default: run local CI equivalent
all: check lint test

# Compile debug binary
build:
	cargo build

# Fast type-check (no codegen)
check:
	cargo check --all-targets

# Apply formatting
fmt:
	cargo fmt --all

# Run clippy lints
lint:
	cargo clippy --all-targets -- -D warnings

# Run all tests (unit + integration)
test:
	cargo test

# Remove build artifacts
clean:
	cargo clean
