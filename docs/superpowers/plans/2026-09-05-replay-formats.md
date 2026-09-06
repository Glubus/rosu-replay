# Stable and lazer replay implementation plan

**Goal:** Separate stable/lazer models while preserving replay contents and rejecting corrupt input.
**Architecture:** Public Replay enum, private format/version fields, ReplayCommon, shared binary codec. Raw lazer JSON mods preserve unknown settings; typed conversion requires the replay mode. Statistics preserve unknown judgement names.

## Tasks
- [x] Add independent wire-format contract tests; run against the PR to demonstrate failures.
- [x] Introduce stable/lazer models, validated version types and shared codec; migrate existing callers.
- [x] Preserve lazer metadata and mode-specific mods; handle signed IDs, null/empty blocks, truncation and bounded decompression.
- [x] Add full fixture comparisons, version boundary tables, four-mode cases and malformed-input tests. Remove tests that silently skip failures.
- [x] Document the breaking API and diagnostic-only uncompressed output. Run fmt, default/all-feature tests, clippy and check available WASM tooling.

## Validation contracts
Tests construct binary headers with byteorder and LZMA payloads directly, never with Packer. Re-encoded suffixes are decoded independently. Assertions cover complete JSON, frames, seed, signed ID and exact suffix width. Test changes must detect data loss, wrong version branches, invalid mode inference, swallowed IO errors or excessive allocations.

## Files
`src/replay.rs`: common model, enum and IO entry points. `src/stable/`: stable model and version capabilities. `src/lazer/`: lazer model/version, statistics and lossless mods. `src/codec/`: shared reader, writer and compression. Compatibility reexports for packer/unpacker remain. `tests/format_contracts.rs` and `tests/support/`: independent format oracle. Existing tests/examples/WASM wrapper migrate to explicit common accessors.

## Validation results

- The initial seven contract tests all failed on 289cead, then passed after the rewrite.
- 64 tests pass with all features (including 22 format contracts and 3 doctests).
- Default-feature tests, Clippy with warnings denied, rustfmt and rustdoc pass.
- wasm32-unknown-unknown check and full build pass using the already installed LLVM Clang/llvm-ar; no browser runtime execution claimed.
