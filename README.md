# rosu-replay

[![Crates.io](https://img.shields.io/crates/v/rosu-replay.svg)](https://crates.io/crates/rosu-replay)
[![Documentation](https://docs.rs/rosu-replay/badge.svg)](https://docs.rs/rosu-replay)
[![License](https://img.shields.io/crates/l/rosu-replay.svg)](LICENSE)

A Rust library for parsing and writing osu! replay files (.osr format).

This library is a **faithful port of the Python [`osrparse`](https://github.com/kszlim/osu-replay-parser) library**, providing the same functionality for parsing and manipulating osu! replay files in Rust with improved performance and memory safety.

## Features

- 🎮 **Parse .osr replay files** from disk or memory
- 📊 **Extract complete replay metadata** including:
  - Player information (username, score, combo, etc.)
  - Game metadata (mode, mods, timestamp, etc.)
  - Hit statistics (300s, 100s, 50s, misses, etc.)
  - Replay events (cursor movement, key presses)
  - Life bar data over time
- 💾 **Write replay files** back to .osr format
- 🎯 **Support all game modes**: osu!standard, osu!taiko, osu!catch, osu!mania
- 🌐 **API compatibility** for parsing replay data from osu! API v1 responses
- ⚡ **High performance** with zero-copy parsing where possible
- 🦀 **Memory safe** Rust implementation
- 📖 **Comprehensive documentation** and examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rosu-replay = "0.1"
```

## Quick Start

### Parsing a Replay File

```rust
use rosu_replay::Replay;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a replay file
    let replay = Replay::from_path("path/to/replay.osr")?;
    
    // Access basic information
    println!("Player: {}", replay.username);
    println!("Score: {}", replay.score);
    println!("Max Combo: {}", replay.max_combo);
    println!("Game Mode: {:?}", replay.mode);
    println!("Mods: {:?}", replay.mods);
    
    // Access hit statistics
    println!("300s: {}, 100s: {}, 50s: {}, Misses: {}", 
        replay.count_300, replay.count_100, replay.count_50, replay.count_miss);
    
    Ok(())
}
```

### Working with Replay Events

```rust
use rosu_replay::{Replay, ReplayEvent};

let replay = Replay::from_path("replay.osr")?;

// Iterate through replay events
for (i, event) in replay.replay_data.iter().enumerate().take(10) {
    match event {
        ReplayEvent::Osu(osu_event) => {
            println!("Frame {}: Cursor at ({:.1}, {:.1}) at +{}ms, keys: {}", 
                i, osu_event.x, osu_event.y, osu_event.time_delta, osu_event.keys.value());
        }
        ReplayEvent::Taiko(taiko_event) => {
            println!("Frame {}: Taiko input at +{}ms, keys: {}", 
                i, taiko_event.time_delta, taiko_event.keys.value());
        }
        ReplayEvent::Catch(catch_event) => {
            println!("Frame {}: Catch at x={:.1}, +{}ms, dashing: {}", 
                i, catch_event.x, catch_event.time_delta, catch_event.dashing);
        }
        ReplayEvent::Mania(mania_event) => {
            println!("Frame {}: Mania keys {} at +{}ms", 
                i, mania_event.keys.value(), mania_event.time_delta);
        }
    }
}
```

### Modifying and Writing Replays

```rust
use rosu_replay::Replay;

let mut replay = Replay::from_path("input.osr")?;

// Modify replay data
replay.username = "Modified Player".to_string();
replay.score = 1000000;

// Write to a new file
replay.write_path("modified_replay.osr")?;
```

### Working with API Data

```rust
use rosu_replay::{parse_replay_data, GameMode};
use base64::{Engine as _, engine::general_purpose};

// Example: parsing replay data from osu! API v1
let api_response = "base64_encoded_replay_data_from_api";
let decoded_data = general_purpose::STANDARD.decode(api_response)?;

// Parse the replay events
let events = parse_replay_data(&decoded_data, true, false, GameMode::Std)?;

for event in events.iter().take(5) {
    if let rosu_replay::ReplayEvent::Osu(osu_event) = event {
        println!("API Event: ({:.1}, {:.1}) +{}ms", 
            osu_event.x, osu_event.y, osu_event.time_delta);
    }
}
```

## Examples

Check out the [`examples/`](examples/) directory for more comprehensive usage examples:

```bash
# Run the basic example (requires test.osr in assets/ directory)
cargo run --example example_1

# Generate documentation with examples
cargo doc --open
```

## Game Mode Support

This library supports all osu! game modes:

| Mode | Enum | Description |
|------|------|-------------|
| osu!standard | `GameMode::Std` | Traditional circle-clicking mode |
| osu!taiko | `GameMode::Taiko` | Drum-based rhythm mode |
| osu!catch | `GameMode::Catch` | Fruit-catching mode |
| osu!mania | `GameMode::Mania` | Piano-style rhythm mode |

Each mode has its own event type with mode-specific data:
- `ReplayEventOsu`: x/y coordinates and key states
- `ReplayEventTaiko`: drum hit positions and types
- `ReplayEventCatch`: horizontal position and dash state
- `ReplayEventMania`: key states for multiple lanes

## File Format

The .osr format is a binary format used by osu! to store replay data. This library handles:

- All metadata fields (player, score, mods, etc.)
- LZMA-compressed replay event data
- Life bar data
- Timestamp conversion between Windows ticks and Unix timestamps
- Both old (32-bit) and new (64-bit) replay ID formats

## Attribution

This library is a port of the excellent Python [`osrparse`](https://github.com/kszlim/osu-replay-parser) library by [kszlim](https://github.com/kszlim) and contributors. The original Python implementation provided the foundation for understanding the .osr file format and replay data structures.

**Original Python library**: https://github.com/kszlim/osu-replay-parser

While this Rust port maintains API compatibility where possible, it leverages Rust's type system and memory safety features to provide additional guarantees and performance improvements.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **kszlim** and contributors for the original [`osrparse`](https://github.com/kszlim/osu-replay-parser) Python library
- The osu! community for documenting the .osr file format
- **ppy** for creating osu! and maintaining the replay format
