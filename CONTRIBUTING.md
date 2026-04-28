# Contributing to otari (Rust)

Thank you for your interest in contributing to otari!

We're building a simple, unified Rust interface for working with LLMs through the Otari gateway, and we welcome contributions from developers of all experience levels.

## Before You Start

### Check for Duplicates

Before creating a new issue or starting work:
- [ ] Search [existing issues](https://github.com/mozilla-ai/otari-sdk-rust/issues) for duplicates
- [ ] Check [open pull requests](https://github.com/mozilla-ai/otari-sdk-rust/pulls) to see if someone is already working on it
- [ ] For bugs, verify it still exists in the `main` branch

### Discuss Major Changes First

For significant changes, please open an issue **before** starting work:

- API changes or new public methods
- Breaking changes
- New dependencies

## Development Setup

### Prerequisites

- **Rust 1.83 or newer** (`rustup update stable`)
- **Git**
- A running **Otari gateway** instance for integration tests

### Quick Start

```bash
# 1. Fork the repository on GitHub

# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/otari-sdk-rust.git
cd otari-sdk-rust

# 3. Add upstream remote
git remote add upstream https://github.com/mozilla-ai/otari-sdk-rust.git

# 4. Build the project
cargo build

# 5. Run tests
cargo test

# 6. Run lints
cargo clippy --all-features --all-targets -- -D warnings

# 7. Format code
cargo fmt
```

### Setting Up API Keys

Create a `.env` file in the project root (this file is gitignored):

```bash
OTARI_API_KEY=your_key_here
OTARI_API_BASE=http://localhost:8000
```

Or export environment variables:

```bash
export OTARI_API_KEY="your_key_here"
export OTARI_API_BASE="http://localhost:8000"
```

**Never commit API keys!**

## Making Changes

### 1. Create a Branch

Always work on a feature branch:

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a new branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code improvements

### 2. Code Style

We use strict Clippy lints. Before committing:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features --all-targets -- -D warnings

# Run tests
cargo test --all-features
```

### 3. Write Tests

**Every change needs tests!**

- **New features**: Add tests covering happy path and error cases
- **Bug fixes**: Add a test that reproduces the bug

Tests are organized as:
- `src/` - Unit tests in `#[cfg(test)]` modules
- `tests/` - Integration tests

### 4. Documentation

Update documentation when you:
- Add a new feature
- Change existing behavior

Documentation to update:
- **Doc comments** (`///`) on public items (required)
- **README.md** if changing core functionality
- **Examples** in `examples/` directory

### 5. Commit Messages

Write clear, descriptive commit messages:

```bash
# Good
git commit -m "feat: add batch operation support"
git commit -m "fix: handle streaming disconnect gracefully"

# Less helpful (avoid)
git commit -m "fix bug"
git commit -m "update"
```

## Submitting Your Contribution

### 1. Push to Your Fork

```bash
git add .
git commit -m "feat: add support for new feature"
git push origin feature/your-feature-name
```

### 2. Create a Pull Request

1. Go to the repository on GitHub
2. Click "New Pull Request"
3. Select your fork and branch
4. Fill out the PR description completely
5. Ensure CI passes

## Review Process

### What to Expect

1. **Initial Response**: Within 5 business days
2. **Simple Fixes**: Usually merged within 1 week
3. **Complex Features**: May take 2-3 weeks

### During Review

- Maintainers will provide constructive feedback
- Address comments with new commits
- CI must pass before merge

## Your First Contribution

New to the project? Look for issues labeled:
- `good-first-issue` - Perfect for newcomers
- `help-wanted` - Community contributions welcome
- `documentation` - Often accessible for beginners

### Starter Ideas

- Fix a typo in documentation
- Add a test case
- Improve error messages
- Add an example

## Development Commands

```bash
# Build
cargo build
cargo build --all-features

# Test
cargo test                          # All tests
cargo test --lib                    # Unit tests only
cargo test --test '*'               # Integration tests only
cargo test test_name                # Specific test

# Lint
cargo clippy --all-features --all-targets -- -D warnings

# Format
cargo fmt
cargo fmt --check                   # Check only

# Documentation
cargo doc --all-features --no-deps --open

# Run example
cargo run --example gateway_completion
```

## Questions?

- Open a GitHub Issue for bugs or feature requests
- Start a Discussion for questions

We're excited to have you contribute!

---

**License**: By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
