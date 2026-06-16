# Changelog

## [0.2.0](https://github.com/mozilla-ai/otari-sdk-rust/compare/otari-0.1.1...otari-0.2.0) (2026-06-16)


### Features

* add image generation and audio (speech/transcription) methods ([#43](https://github.com/mozilla-ai/otari-sdk-rust/issues/43)) ([3372ef6](https://github.com/mozilla-ai/otari-sdk-rust/commit/3372ef65ba76b423c4ca83bb2b50c3cadc7ddcc1))

## [0.1.1](https://github.com/mozilla-ai/otari-sdk-rust/compare/otari-0.1.0...otari-0.1.1) (2026-06-12)


### Features

* add batch API support for gateway provider ([8843730](https://github.com/mozilla-ai/otari-sdk-rust/commit/8843730983ac1eea21725761d1c129e8c4ee1a5c))
* add batch API support for gateway provider ([b9d3c8c](https://github.com/mozilla-ai/otari-sdk-rust/commit/b9d3c8c86e4f0a4f2258b6e663ecca8e1ffd9506))
* add moderation API support to Gateway provider ([42bec35](https://github.com/mozilla-ai/otari-sdk-rust/commit/42bec35d826a97e356c81686d2e8c81ad66e2101))
* add rerank endpoint support ([56a9f5e](https://github.com/mozilla-ai/otari-sdk-rust/commit/56a9f5e91cf57d224d9ef1f534f4a65524572a5f))
* add rerank() free function, RerankOptions, and public re-exports ([8c8b559](https://github.com/mozilla-ai/otari-sdk-rust/commit/8c8b559fb273f7386016ef8468c671b1a7452b57))
* add RerankParams, RerankResult, RerankMeta, RerankUsage, and RerankResponse types ([feacfd0](https://github.com/mozilla-ai/otari-sdk-rust/commit/feacfd0e343bd91ac8b9a28b7402aa29b16fd44c))
* add SUPPORTS_RERANK constant and rerank/rerank_fn methods to Provider trait ([9d3bfef](https://github.com/mozilla-ai/otari-sdk-rust/commit/9d3bfefdc8279509ccf4aed9d8e6514b83edc6d8))
* **gateway:** add gateway provider for any-llm gateway server ([f500878](https://github.com/mozilla-ai/otari-sdk-rust/commit/f500878bb3a15ba4d2ef590f36502166194852eb))
* **gateway:** add gateway provider for any-llm gateway server ([126fae3](https://github.com/mozilla-ai/otari-sdk-rust/commit/126fae3f8cd5817e35b1d3021698425117096e65))
* implement rerank endpoint on gateway provider ([7ea11b4](https://github.com/mozilla-ai/otari-sdk-rust/commit/7ea11b4312f89d03c65bffd7cef3239580e9ef36))
* independent release automation + surface gateway spec version ([#39](https://github.com/mozilla-ai/otari-sdk-rust/issues/39)) ([73d78a5](https://github.com/mozilla-ai/otari-sdk-rust/commit/73d78a567eacd06cb23524b494644a7a7d289c59))
* **refactor:** Strongly typed providers ([fba6b06](https://github.com/mozilla-ai/otari-sdk-rust/commit/fba6b0605d56a1c68ec3f76417df92c2615a394b))
* Strongly Typed Providers ([fba6b06](https://github.com/mozilla-ai/otari-sdk-rust/commit/fba6b0605d56a1c68ec3f76417df92c2615a394b))
* wrap /v1/messages/count_tokens (regenerate core + ergonomic method) ([#35](https://github.com/mozilla-ai/otari-sdk-rust/issues/35)) ([f0c82bd](https://github.com/mozilla-ai/otari-sdk-rust/commit/f0c82bdb80d06d7eb1b116bd96162d14ae84329c))


### Bug Fixes

* address CI clippy failures ([5285b7b](https://github.com/mozilla-ai/otari-sdk-rust/commit/5285b7b2a3dcd36b61e23b3393446a0c4f8a7417))
* address Copilot review feedback ([ea52512](https://github.com/mozilla-ai/otari-sdk-rust/commit/ea525122df66d50aebc72e6c9d5e4e1b2b4120f8))
* **ci:** generate lockfile in MSRV job for compatible dependency resolution ([a92628e](https://github.com/mozilla-ai/otari-sdk-rust/commit/a92628ea35ad30e69eb0f9b228046f4dd51ab253))
* **ci:** make live integration tests skip without a configured gateway ([#36](https://github.com/mozilla-ai/otari-sdk-rust/issues/36)) ([d27855a](https://github.com/mozilla-ai/otari-sdk-rust/commit/d27855a68daf1d949ee46c9c29a9973dc3835597))
* **ci:** use stable resolver fallback to generate MSRV-compatible lockfile ([cbfa151](https://github.com/mozilla-ai/otari-sdk-rust/commit/cbfa1516b3b5f1c31c3c733d4bd21b0045642267))
* inline generated core as a module so the crate publishes to crates.io ([#37](https://github.com/mozilla-ai/otari-sdk-rust/issues/37)) ([1cab254](https://github.com/mozilla-ai/otari-sdk-rust/commit/1cab25495e98649480f834959c232c18d346f438))
* parse batch 409 error using actual gateway message format ([0e2ff33](https://github.com/mozilla-ai/otari-sdk-rust/commit/0e2ff33841004611fc026b45f6d7aaa32a5187e6))
* regenerate SDK client core so message.reasoning is a string ([#41](https://github.com/mozilla-ai/otari-sdk-rust/issues/41)) ([7eb6841](https://github.com/mozilla-ai/otari-sdk-rust/commit/7eb684193a88559f256f44c07b8ad30d59152c46))
* rename gateway auth header from X-AnyLLM-Key to AnyLLM-Key ([e3d2afc](https://github.com/mozilla-ai/otari-sdk-rust/commit/e3d2afc1447ade2519c35bb6563938384d713581))
* resolve clippy float_cmp and assertions_on_constants warnings in rerank tests ([ab2e06a](https://github.com/mozilla-ai/otari-sdk-rust/commit/ab2e06adc50d54a1b7538b1e04946996c984d363))
