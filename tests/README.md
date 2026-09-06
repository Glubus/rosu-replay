# Test strategy

- `format_contracts.rs`: hand-built binary headers and LZMA blocks, version/ID
  widths, full statistics and future metadata, signed seeds, ruleset-specific
  mods, truncation, IO failures and configured resource limits.
- `integration_tests.rs`: real files embedded with `include_bytes!`, full model
  equality, independent JSON decode before the first production parse, known
  header values, presets, fragmented readers and failing streams.
- `wasm_tests.rs`: Rust-side bindings preserve native stable/lazer data; these
  run natively with `--all-features`, not in a browser.
- `parsing_tests.rs`, `api_tests.rs`, `error_tests.rs`: text/API and primitive
  decoding contracts.

`support` deliberately uses byteorder and liblzma directly. It must not call the
production Packer to construct expected binary input, or the production Unpacker
to inspect output. Otherwise matching reader/writer errors can cancel out.

The original PR fails all seven initial format regressions. Tests added during
the rewrite also cover negative seeds, raw zero IDs, four modes, invalid presets,
version invariants and decompression limits. Exact compression bytes, collection
implementation choices and error wording are not test contracts.
