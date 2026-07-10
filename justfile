# Show all available recipes
[private]
default:
    @just --list

# Format the codebase
[group('code')]
format:
    cargo +nightly fmt --all
    just --fmt

# Build debug version
[group('code')]
build:
    cargo build --workspace

# Build release version
[group('code')]
release:
    cargo build --workspace --release

# Run the application
[group('code')]
run *args:
    cargo run -- {{ args }}

# Remove build artifacts
[group('code')]
clean:
    cargo clean

# Check code formatting without modifying files
[group('ci')]
fmt:
    cargo +nightly fmt --all --check
    just --fmt --check

# Check the codebase
[group('ci')]
check:
    cargo check --workspace --all-targets

# Run tests; additional arguments are passed to cargo test
[group('ci')]
test *args:
    cargo test --workspace {{ args }}

# Run Clippy
[group('ci')]
lint:
    cargo clippy --workspace --all-targets --no-deps -- -D warnings

# Configure Git to use the repository hooks
[group('ci')]
setup-hooks:
    git config core.hooksPath .githooks

# Run all pre-commit checks
[group('ci')]
pre-commit: fmt check lint test
    @echo "All checks passed"
