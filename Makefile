.PHONY: test lint format

# Run tests
test:
	cargo test --verbose

# Run Clippy for linting & Check formatting
lint:
	cargo clippy -- -D warnings
	cargo fmt -- --check

# Format code
format:
	cargo fmt
