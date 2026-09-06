# rosu-replay

Read and write osu! `.osr` replays in Rust, including stable and lazer exports.

## Formats and API

`Replay` is an enum with `Stable(StableReplay)` and `Lazer(LazerReplay)` variants.
Both formats share binary IO and LZMA compression. Lazer's `.osr` export stores
an additional LZMA-compressed JSON block from format version `30000001` onward.
The format version is distinct from the lazer client build string.

```rust,no_run
use rosu_replay::Replay;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut replay = Replay::from_path("replay.osr")?;
    println!("{}: {} frames", replay.common().username, replay.common().replay_data.len());
    replay.common_mut().username = "EditedPlayer".into();

    match &replay {
        Replay::Stable(stable) => {
            println!("Legacy mods: {:?}, ID: {:?}", stable.mods(), stable.online_id());
        }
        Replay::Lazer(lazer) => {
            if let Some(info) = lazer.score_info() {
                println!("Client: {}, lazer ID: {}", info.client_version, info.online_id);
                println!("Mods for this ruleset: {:?}", lazer.mods()?);
            }
        }
    }
    replay.write_path("edited.osr")?;
    Ok(())
}
```

`StableReplay::new` and `LazerReplay::new` take `ReplayCommon`, a validated
`StableVersion`/`LazerVersion`, and their format-specific fields. Constructors,
serde deserialization and writing check version-dependent invariants. Editing
common data is supported; writing rejects frames from a different ruleset.

## Lazer metadata

- `LazerScoreInfo` uses signed IDs and a signed 64-bit score without mods.
- `HitStatistics` stores sparse `HitResult` counts, including unknown names.
  `statistics.count(&HitResult::Perfect)` returns zero when absent.
- `LazerMod` preserves raw settings and additional fields, including settings
  not understood by the installed `rosu-mods` version.
- `LazerReplay::mods()` offers a typed `rosu_mods::GameMods` view using the
  replay's mode. It can return an error for settings the dependency cannot
  interpret; the raw metadata remains available and can still be written.
- Additional score JSON fields are retained in `LazerScoreInfo::extra`.

Round trips preserve semantic contents, not compressed byte identity or JSON
formatting. Empty/null score blocks are written as a zero-length block. Optional
null JSON values may be omitted; missing default-valued fields may be emitted.
Legacy online IDs preserve zero and negative sentinels; `None` means the old
stable version has no ID field. Lazer's legacy ID and JSON online ID are separate.

## Input limits and errors

`Replay::from_reader_with_limits(reader, ReadLimits { .. })` allows callers to
adjust size limits. Defaults: strings 16 MiB, compressed blocks 64 MiB,
decompressed frames 256 MiB, decompressed score JSON 16 MiB. LZMA decoder memory
is independently capped at 256 MiB. Forged lengths do not trigger eager allocations.
Truncated blocks and IO failures are errors, including truncated lazer lengths.

`Packer::new().with_preset(0..=9)` selects the LZMA compression preset.
`pack_uncompressed()` is retained only for diagnostic dumps: its output is **not
valid `.osr`** and cannot be read by the ordinary replay reader. Lazer metadata
is still included in that diagnostic output.

## API replay data

For the frames-only response from osu! API v1, use
`parse_replay_data(data, decoded, decompressed, mode)`. The booleans indicate
whether base64 decoding and decompression have already been performed.

## Migration from 0.2.2

This source change breaks the former flat `Replay` struct API:

| Previous access | New access |
| --- | --- |
| `replay.username`, `replay.replay_data` | `replay.common().username`, `replay.common().replay_data` |
| Assigning common fields | `replay.common_mut()` |
| `replay.game_version` | `replay.game_version()` |
| `replay.mods` (legacy bitflags) | `replay.legacy_mods()` |
| `replay.replay_id` | `replay.legacy_online_id()` (raw `Option<i64>`) |
| `replay.lazer_score_info` | Match `Replay::Lazer`, then `score_info()` / `score_info_mut()` |
| Typed lazer mods | `lazer.mods()?` |
| `LazerScoreInfoStatistics` | `HitStatistics` with all known and unknown judgements |
| `Replay { ... }` literal | Format-specific constructor, then `.into()` |

The old `packer::Packer` and `unpacker::Unpacker` import paths remain available;
the shared implementation lives in `codec`. Public types are reexported at the
crate root. Serde's representation of `Replay` is now tagged by variant.

## Verification

```sh
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run --example example_1 -- assets/test_lazer.osr
```

Contract tests build binary inputs independently of the production writer, check
version boundaries and signed values, decode rewritten JSON independently, and
exercise real stable/lazer fixtures. Fixture failures fail tests rather than skip.
The `wasm` feature exposes Rust bindings; native binding tests do not constitute
a browser/WASM runtime test. Cross-compilation was also checked with
`cargo check` and `cargo build --target wasm32-unknown-unknown --features wasm`; liblzma-sys
requires a Clang compiler for this target.

## License

MIT. Originally ported from Python's `osrparse`.
