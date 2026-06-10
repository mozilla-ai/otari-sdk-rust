//! Shared helpers for the `#[ignore]` live-gateway integration tests.

use otari::{Config, Otari};

/// Build a client for a live (`#[ignore]`) gateway test, or `None` when no
/// gateway is configured.
///
/// The live tests reach a real gateway through the `OTARI_API_BASE` /
/// `OTARI_API_KEY` environment variables (see `Otari::from_config`). When
/// `OTARI_API_BASE` is unset or empty (for example in CI without a configured
/// gateway), this returns `None` so the caller can skip instead of panicking on
/// an empty base URL.
pub fn live_client() -> Option<Otari> {
    match std::env::var("OTARI_API_BASE") {
        Ok(base) if !base.trim().is_empty() => Otari::from_config(Config::default()).ok(),
        _ => {
            eprintln!("skipping live gateway test: OTARI_API_BASE is not set");
            None
        }
    }
}
