//! OpenAPI-generated typed core. This module is produced by the otari codegen
//! pipeline (scripts/sdk_codegen) and is not hand-edited; the lint allowances
//! below exempt it from the SDK crate's strict lints, which it escaped when it
//! lived in its own crate. Do not add hand-written code here.
#![allow(unused_imports)]
#![allow(dead_code)]
// Schemas whose names carry their defining module (Pydantic disambiguates
// duplicate class names that way, e.g. anthropic__types__thinking_block__
// ThinkingBlock) generate module names with double underscores, which rustc's
// non_snake_case lint rejects under the SDK crate's -D warnings.
#![allow(non_snake_case)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

pub mod apis;
pub mod models;
pub mod spec_version;
