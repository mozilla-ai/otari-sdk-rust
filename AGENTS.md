# Repository Guidelines

## Where to Look First

- [README.md](README.md): High-level usage and quick start.
- [CONTRIBUTING.md](CONTRIBUTING.md): Canonical dev setup, test matrix, and contribution workflow.
- [Cargo.toml](Cargo.toml): Dependencies, features, and lint configuration.
- [examples/](examples/): Working code examples.

## Project Structure & Module Organization

- `src/`: Library source code
  - `lib.rs`: Public API exports and crate-level documentation
  - `api.rs`: High-level functions (`completion()`, `completion_stream()`)
  - `error.rs`: Unified error types (`AnyLLMError`)
  - `provider.rs`: `Provider` trait and factory function
  - `providers/`: Provider implementations
    - `openai.rs`: OpenAI via `async-openai` SDK
    - `anthropic.rs`: Anthropic via `reqwest` + SSE
  - `types/`: Shared data types (messages, completions, tools, streaming chunks)
- `tests/`: Test suites
  - `test_*.rs`: Unit tests for each module
  - `integration_*.rs`: Integration tests (require API keys)
- `examples/`: Runnable examples demonstrating usage

## Build, Test, and Development Commands

This repo uses `cargo` (Rust 1.83+). For the full command set, see [CONTRIBUTING.md](CONTRIBUTING.md).

- Build: `cargo build --all-features`
- Run all checks: `cargo fmt --check && cargo clippy --all-features -- -D warnings`
- Unit tests: `cargo test --all-features`
- Run example: `cargo run --example basic_completion`
- Build docs: `cargo doc --all-features --no-deps --open`

## Coding Style & Naming Conventions

- Rust indentation: 4 spaces (default)
- Formatting via `rustfmt` (config in `rustfmt.toml`)
- Linting via `clippy` with pedantic + nursery lints enabled (see `Cargo.toml` `[lints.clippy]`)
- Provider code lives under `src/providers/<provider>.rs`
- Public items require doc comments (`///`)
- Add code comments only where logic isn't self-evident; remove obvious comments before finishing

## Testing Guidelines

- Framework: Built-in Rust test framework + `tokio::test` for async
- Add/adjust tests with every change (happy path + error cases)
- Integration tests should skip gracefully when API keys aren't available
- New code should have reasonable test coverage

## Commit & Pull Request Guidelines

- Commits: Use descriptive messages like `feat: add streaming support`, `fix: handle rate limit errors`
- PRs should follow the pull request template: clear description, linked issues, completed checklist

## Security & Configuration Tips

- Never commit secrets. Use environment variables or a local `.env` (gitignored) for provider API keys.
- Required env vars: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`
