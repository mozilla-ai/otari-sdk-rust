.PHONY: build
build:
	cargo build --all-features

.PHONY: format
format:
	@echo "Formatting..."
	@cargo fmt

.PHONY: lint
lint:
	cargo clippy \
		--all-features \
		--all-targets \
		-- \
		-D warnings

.PHONY: fix
fix:
	@cargo clippy \
		--fix \
		--all-features \
		--allow-dirty

.PHONY: test
test:
	cargo test --all-features

.PHONY: test-unit
test-unit:
	cargo test --lib --all-features

.PHONY: test-integration
test-integration:
	cargo test --test '*' --all-features

.PHONY: check
check:
	cargo check --all-features

.PHONY: docs
docs:
	cargo doc --all-features --no-deps --open

.PHONY: clean
clean:
	cargo clean

.PHONY: all
all: format lint test
