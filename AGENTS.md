# Repository Guidelines

`CLAUDE.md` is a symlink to this file. Always edit `AGENTS.md` directly; never modify `CLAUDE.md`.

## Architecture (Big Picture)

This SDK is a **thin, hand-written shell over an OpenAPI-generated typed core**.

- **Generated core: the `otari-client` crate in `client/`** (path dependency:
  `otari-client = { path = "client" }`): produced by OpenAPI Generator from the gateway's
  OpenAPI spec. It is **generated, not hand-edited.** Regeneration happens upstream in the gateway
  repo (`.github/workflows/gateway-sdk-codegen.yml`), which opens a `sdk-codegen/client-core` PR
  here. Because `client/` is an unpublished path dependency, releasing `otari` to crates.io
  requires handling it (publish `otari-client` first, or vendor); see `release.yml`.
- **Hand-written shell** (`src/`): ergonomic API + auth + streaming + typed errors over that core.
  - `core.rs` is the key seam: `map_error()` / `map_response()` convert gateway HTTP responses
    into the typed `OtariError`.
  - `src/client/models/stream.rs` implements SSE streaming via `reqwest-eventsource` (the
    generated core cannot stream).

### Two auth modes (must both keep working)
- **Platform** (`OTARI_AI_TOKEN`): `Authorization: Bearer`, base URL defaults to the hosted gateway.
- **Self-hosted** (`GATEWAY_API_KEY` + `GATEWAY_API_BASE`, legacy aliases `OTARI_API_KEY` /
  `OTARI_API_BASE`): `Otari-Key` header. Typed error mapping applies in both modes.

### Endpoint-coverage drift gate
`tests/endpoint_coverage.rs` fetches the canonical gateway spec
(`https://raw.githubusercontent.com/mozilla-ai/otari/main/docs/public/openapi.json`) and asserts
every endpoint is accounted for in `sdk-endpoints.txt` (`[covered]` / `[excluded]`). Update that
manifest when you add or intentionally skip an endpoint.

## Where to Look First

- [README.md](README.md): High-level usage and quick start.
- [CONTRIBUTING.md](CONTRIBUTING.md): Canonical dev setup, test matrix, and contribution workflow.
- [Cargo.toml](Cargo.toml): Dependencies, features, and lint configuration.
- [examples/](examples/): Working code examples.

## Project Structure & Module Organization

- `src/`: Library source code
  - `lib.rs`: Public API exports and crate-level documentation
  - `api.rs`: High-level functions (`completion()`, `completion_stream()`)
  - `core.rs`: Generated-core configuration and error mapping (`map_error()` / `map_response()`)
  - `error.rs`: Unified error types (`OtariError`)
  - `config.rs`: `Config` struct for client configuration
  - `client/`: hand-written `Otari` client shell (auth modes, endpoint methods) over `otari-client`
    - `client/models/stream.rs`: SSE streaming via `reqwest-eventsource`
  - `control_plane.rs`: control-plane API wrappers (keys/users/budgets/pricing/usage)
  - `types/`: Shared data types (messages, completions, tools, streaming chunks, batch, moderation, rerank)
- `client/`: the generated `otari-client` crate (OpenAPI output; do not hand-edit)
- `sdk-endpoints.txt`: endpoint-coverage manifest gated by `tests/endpoint_coverage.rs`
- `tests/`: Test suites
  - `test_*.rs`: Unit tests for each module
  - `integration_*.rs`: Integration tests (require a running gateway)
- `examples/`: Runnable examples demonstrating usage

## Build, Test, and Development Commands

This repo uses `cargo` (Rust 1.83+). For the full command set, see [CONTRIBUTING.md](CONTRIBUTING.md).

- Build: `cargo build --all-features`
- Run all checks: `cargo fmt --all -- --check && cargo clippy --all-features --all-targets -- -D warnings`
- Unit tests: `cargo test --all-features`
- Drift gate: `cargo test --all-features --test endpoint_coverage`
- Integration tests (need a gateway + keys): `cargo test --all-features -- --ignored`
- MSRV check (Rust 1.83): `cargo check --all-features --locked`
- Run example: `cargo run --example gateway_completion`
- Build docs: `cargo doc --all-features --no-deps --open` (CI runs with `RUSTDOCFLAGS=-D warnings`)

## Coding Style & Naming Conventions

- Rust indentation: 4 spaces (default)
- Formatting via `rustfmt` (config in `rustfmt.toml`)
- Linting via `clippy` with pedantic + nursery lints enabled (see `Cargo.toml` `[lints.clippy]`)
- Client code lives under `src/client/`
- Public items require doc comments (`///`)
- Add code comments only where logic isn't self-evident; remove obvious comments before finishing

## Testing Guidelines

- Framework: Built-in Rust test framework + `tokio::test` for async
- Add/adjust tests with every change (happy path + error cases)
- Integration tests should skip gracefully when the gateway is not available
- New code should have reasonable test coverage

## Commit & Pull Request Guidelines

- Commits: Use descriptive messages like `feat: add streaming support`, `fix: handle rate limit errors`
- PRs should follow the pull request template: clear description, linked issues, completed checklist

## Security & Configuration Tips

- Never commit secrets. Use environment variables or a local `.env` (gitignored) for API keys.
- Platform mode: `OTARI_AI_TOKEN`. Self-hosted mode: `GATEWAY_API_KEY` + `GATEWAY_API_BASE`
  (legacy aliases `OTARI_API_KEY` / `OTARI_API_BASE` are still read as a fallback).

## Writing style

- Avoid em dashes and double hyphens (`--`) used as separators in prose
  (README, docs, doc comments, commit messages, PR descriptions). Use commas,
  semicolons, colons, parentheses, or periods, or rephrase. This does not apply
  to code (for example CLI flags like `--all`) or en-dash numeric ranges like `3–4`.

## Notes for Agents

- Never hand-edit the generated `client/` (`otari-client`) crate; it is regenerated from the
  gateway spec. Fix the shell, or the upstream spec/generator, instead.
- Added/removed an endpoint wrapper → update `sdk-endpoints.txt` and run the drift gate.
