//! Endpoint-coverage drift gate.
//!
//! Fetches the canonical otari gateway OpenAPI spec and asserts that every API
//! endpoint it exposes is accounted for in `sdk-endpoints.txt` -- either wrapped
//! by this SDK's public surface (`[covered]`) or deliberately deferred
//! (`[excluded]`). A new gateway endpoint in neither section fails this test, so
//! a future endpoint (as `/messages` once was) cannot silently go unsurfaced.
//!
//! The fetch uses `reqwest` (already a dependency). Offline it is a soft pass
//! (logged and skipped) unless `CI` is set, where the network is available and a
//! fetch failure is a hard error. `OTARI_SKIP_NETWORK_TESTS=1` forces the skip.

use std::collections::BTreeSet;

const SPEC_URL: &str =
    "https://raw.githubusercontent.com/mozilla-ai/otari/main/docs/public/openapi.json";
const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

/// Parse the manifest into `(covered, excluded)` endpoint sets.
fn parse_manifest() -> (BTreeSet<String>, BTreeSet<String>) {
    let text = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sdk-endpoints.txt"));
    let mut covered = BTreeSet::new();
    let mut excluded = BTreeSet::new();
    let mut section: Option<&mut BTreeSet<String>> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line {
            "[covered]" => {
                section = Some(&mut covered);
                continue;
            }
            "[excluded]" => {
                section = Some(&mut excluded);
                continue;
            }
            _ => {}
        }
        let entry = line.split('#').next().unwrap_or("").trim();
        if entry.is_empty() {
            continue;
        }
        let mut parts = entry.split_whitespace();
        let (Some(method), Some(path)) = (parts.next(), parts.next()) else {
            continue;
        };
        if let Some(set) = section.as_deref_mut() {
            set.insert(format!("{} {}", method.to_uppercase(), path));
        }
    }
    (covered, excluded)
}

/// Whether the suite should treat an unreachable spec URL as a hard failure.
/// True in CI (where the network is available); a soft skip locally/offline.
fn require_network() -> bool {
    std::env::var("OTARI_SKIP_NETWORK_TESTS").as_deref() != Ok("1") && std::env::var("CI").is_ok()
}

/// Fetch the spec and return its `METHOD /path` set, dropping `/health*` routes.
/// `Ok(None)` means "could not fetch, soft skip"; `Err` means "hard failure".
fn fetch_spec_endpoints() -> Result<Option<BTreeSet<String>>, String> {
    if std::env::var("OTARI_SKIP_NETWORK_TESTS").as_deref() == Ok("1") {
        eprintln!("OTARI_SKIP_NETWORK_TESTS=1: skipping endpoint-coverage network fetch");
        return Ok(None);
    }
    let runtime = tokio::runtime::Runtime::new().map_err(|e| format!("tokio runtime: {e}"))?;
    let fetched = runtime.block_on(async {
        let resp = reqwest::get(SPEC_URL).await?.error_for_status()?;
        resp.text().await
    });
    let body = match fetched {
        Ok(text) => text,
        Err(e) => {
            let msg = format!("could not fetch otari OpenAPI spec from {SPEC_URL}: {e}");
            if require_network() {
                return Err(msg);
            }
            eprintln!("{msg} (offline soft-skip)");
            return Ok(None);
        }
    };
    let doc: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("parsing spec JSON: {e}"))?;
    let paths = doc
        .get("paths")
        .and_then(serde_json::Value::as_object)
        .ok_or("spec has no `paths` object")?;
    let mut eps = BTreeSet::new();
    for (path, methods) in paths {
        if path == "/health" || path.starts_with("/health/") {
            continue;
        }
        if let Some(obj) = methods.as_object() {
            for method in obj.keys() {
                if HTTP_METHODS.contains(&method.to_lowercase().as_str()) {
                    eps.insert(format!("{} {path}", method.to_uppercase()));
                }
            }
        }
    }
    Ok(Some(eps))
}

#[test]
fn manifest_parses_non_empty_and_disjoint() {
    let (covered, excluded) = parse_manifest();
    assert!(!covered.is_empty(), "manifest [covered] section is empty");
    let overlap: Vec<_> = covered.intersection(&excluded).collect();
    assert!(
        overlap.is_empty(),
        "endpoints in both sections: {overlap:?}"
    );
}

#[test]
fn spec_endpoints_are_accounted_for() {
    let (covered, excluded) = parse_manifest();
    let Some(spec) = fetch_spec_endpoints().expect("spec fetch") else {
        return; // soft skip offline
    };
    let accounted: BTreeSet<_> = covered.union(&excluded).cloned().collect();
    let unaccounted: Vec<_> = spec.difference(&accounted).cloned().collect();
    assert!(
        unaccounted.is_empty(),
        "Gateway OpenAPI exposes endpoint(s) the SDK does not account for: {unaccounted:?}. \
         Add a public wrapper and list under [covered], or defer it under [excluded] \
         with a reason, in sdk-endpoints.txt."
    );
}

#[test]
fn manifest_has_no_stale_entries() {
    let (covered, excluded) = parse_manifest();
    let Some(spec) = fetch_spec_endpoints().expect("spec fetch") else {
        return; // soft skip offline
    };
    let accounted: BTreeSet<_> = covered.union(&excluded).cloned().collect();
    let stale: Vec<_> = accounted.difference(&spec).cloned().collect();
    if !stale.is_empty() {
        // Warn-only: stale entries do not fail the build.
        eprintln!("manifest entries not present in current spec (review): {stale:?}");
    }
}
