# Releasing

This SDK is versioned independently of the otari gateway, with its own semver.
Releases are automated with [release-please](https://github.com/googleapis/release-please).

## How a release happens

1. Merge changes to `main` using [Conventional Commits](https://www.conventionalcommits.org/)
   (`feat:`, `fix:`, etc.). This includes the gateway codegen's regeneration PRs
   and ordinary shell PRs.
2. release-please opens or updates a single **release PR** that bumps the version
   in `Cargo.toml` (`[package].version`) and writes `CHANGELOG.md`.
3. Review and merge the release PR. That tags the release and creates a GitHub
   Release.
4. The same workflow run (`.github/workflows/release-please.yml`, gated on
   `release_created`) runs the tests and publishes the crate to crates.io.

## Configuration

- **Registry:** crates.io (`otari`).
- **Auth:** `CARGO_REGISTRY_TOKEN` repository secret.
- **Version file:** `Cargo.toml` `[package].version` (release-please owns it; do
  not edit it by hand). `Cargo.lock` is gitignored, so `cargo publish` runs
  without `--allow-dirty`.

## Prerequisites (one time, repo settings)

- Enable **Settings to Actions: "Allow GitHub Actions to create and approve pull
  requests"** so release-please can open its release PR.
- Store the `CARGO_REGISTRY_TOKEN` secret (a crates.io API token with publish
  rights for the `otari` crate).

## If the publish fails

The release tag and GitHub Release already exist, so re-run the failed
`Publish to crates.io` job from the Actions tab to retry publishing the same
version. Avoid cutting a release by hand; the automated path keeps `Cargo.toml`,
the tag, and the changelog in sync.

See the gateway's [SDK release coordination and compatibility](https://github.com/mozilla-ai/otari/blob/main/docs/sdk-compatibility.md)
for the cross-repo policy, the spec-version model, and the end-to-end flow.
