//! Endpoint-coverage manifest checks.
//!
//! `sdk-endpoints.txt` records which gateway endpoints this SDK surfaces
//! (`[covered]`) and which it deliberately does not (`[excluded]`). The file is
//! a generated artifact: the gateway's codegen workflow pushes it here alongside
//! the generated core, from the canonical copy at
//! `scripts/sdk_codegen/sdk-endpoints.txt` in `mozilla-ai/otari`.
//!
//! The drift gate itself lives in the gateway, where the manifest is validated
//! against `docs/public/openapi.json` from the same commit. It used to live here
//! and fetch the spec from `main` over the network at test time, which made the
//! result depend on when the test ran rather than on what the commit contained:
//! an unchanged commit passed one day and failed the next, and because CI only
//! runs on push and pull_request, `main` sat red unnoticed for over two weeks
//! (mozilla-ai/otari#438). What remains here is offline and deterministic.

use std::collections::BTreeSet;

const HTTP_METHODS: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

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

#[test]
fn manifest_sections_are_non_empty() {
    let (covered, excluded) = parse_manifest();
    assert!(!covered.is_empty(), "manifest [covered] section is empty");
    assert!(!excluded.is_empty(), "manifest [excluded] section is empty");
}

#[test]
fn manifest_sections_are_disjoint() {
    let (covered, excluded) = parse_manifest();
    let both: Vec<_> = covered.intersection(&excluded).cloned().collect();
    assert!(
        both.is_empty(),
        "endpoint(s) in both [covered] and [excluded]: {both:?}"
    );
}

#[test]
fn manifest_entries_are_well_formed() {
    let (covered, excluded) = parse_manifest();
    let malformed: Vec<_> = covered
        .union(&excluded)
        .filter(|entry| {
            let mut parts = entry.splitn(2, ' ');
            let method = parts.next().unwrap_or("");
            let path = parts.next().unwrap_or("");
            !HTTP_METHODS.contains(&method) || !path.starts_with('/')
        })
        .cloned()
        .collect();
    assert!(
        malformed.is_empty(),
        "manifest entries are not \"METHOD /path\": {malformed:?}"
    );
}
