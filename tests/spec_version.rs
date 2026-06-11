//! The gateway/spec version is surfaced at the crate root.
//!
//! The gateway codegen stamps the spec version into the generated core
//! (`_client::spec_version`); the crate re-exports it as `otari::SPEC_VERSION`
//! so callers can tell which gateway spec this SDK targets.

#[test]
fn spec_version_is_surfaced() {
    assert!(!otari::SPEC_VERSION.is_empty());
}
