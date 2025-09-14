# rosu-replay

[![Crates.io](https://img.shields.io/crates/v/rosu-replay.svg)](https://crates.io/crates/rosu-replay)
[![Documentation](https://docs.rs/rosu-replay/badge.svg)](https://docs.rs/rosu-replay)
[![License](https://img.shields.io/crates/l/rosu-replay.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/Glubus/rosu-replay/ci.yml?branch=master)](https://github.com/Glubus/rosu-replay/actions)

A high-performance Rust library for parsing and writing osu! replay files (.osr format), with WebAssembly support.

This library is a **faithful port of the Python [`osrparse`](https://github.com/kszlim/osu-replay-parser) library**, providing the same functionality for parsing and manipulating osu! replay files in Rust with improved performance, memory safety, and additional features.

## ✨ Features

- 🎮 **Parse .osr replay files** from disk, memory, or web
- 📊 **Extract complete replay metadata** including:
  - Player information (username, score, combo, etc.)
  - Game metadata (mode, mods, timestamp, etc.)
  - Hit statistics (300s, 100s, 50s, misses, etc.)
  - Replay events (cursor movement, key presses)
  - Life bar data over time
- 💾 **Write replay files** back to .osr format (compressed and uncompressed)
- 🎯 **Support all game modes**: osu!standard, osu!taiko, osu!catch, osu!mania
- 🌐 **API compatibility** for parsing replay data from osu! API v1 responses
- 🕸️ **WebAssembly support** for browser and Node.js environments
- ⚡ **High performance** with optimized LZMA compression via `liblzma`
- 🦀 **Memory safe** Rust implementation with zero-copy parsing where possible
- 📖 **Comprehensive documentation** and examples
- 🧪 **Extensive testing** with 40+ tests covering all functionality

## 🚀 Installation

### Rust/Cargo

Add this to your `Cargo.toml`:

```toml
[dependencies]
rosu-replay = "0.1"
```

### WebAssembly

For WASM usage, enable the `wasm` feature:

```toml
[dependencies]
rosu-replay = { version = "0.1", features = ["wasm"] }
```

Then compile with:

```bash
wasm-pack build --features wasm --target web
```

## 📖 Quick Start

### Basic Replay Parsing

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
    
    // Check if it's a perfect play
    println!("Perfect: {}", replay.count_miss == 0);
    
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
use rosu_replay::{Replay, Packer};

let mut replay = Replay::from_path("input.osr")?;

// Modify replay data
replay.username = "Modified Player".to_string();
replay.score = 1000000;

// Write compressed (default)
replay.write_path("modified_replay.osr")?;

// Write uncompressed for faster loading
replay.write_path_uncompressed("uncompressed_replay.osr")?;

// Custom compression settings
let custom_packer = Packer::new().with_preset(9); // Maximum compression
let bytes = replay.pack_with(&custom_packer)?;
std::fs::write("custom_compressed.osr", bytes)?;
```

### Working with API Data

```rust
use rosu_replay::{parse_replay_data, GameMode};

// Parse replay data from osu! API v1
let api_data = b"base64_encoded_replay_data_from_api";
let events = parse_replay_data(api_data, false, false, GameMode::Std)?;

for event in events.iter().take(5) {
    if let rosu_replay::ReplayEvent::Osu(osu_event) = event {
        println!("API Event: ({:.1}, {:.1}) +{}ms", 
            osu_event.x, osu_event.y, osu_event.time_delta);
    }
}
```

## 🕸️ WebAssembly Usage

### Browser JavaScript

```javascript
import init, { WasmReplay, WasmGameMode, parse_replay_data_wasm } from './pkg/rosu_replay.js';

async function parseReplay() {
    await init();
    
    // Load replay file (from file input, fetch, etc.)
    const replayBytes = new Uint8Array(await file.arrayBuffer());
    
    // Parse replay
    const replay = new WasmReplay(replayBytes);
    
    console.log(`Player: ${replay.username}`);
    console.log(`Score: ${replay.score}`);
    console.log(`Mode: ${replay.mode}`);
    console.log(`Events: ${replay.event_count}`);
    
    // Pack back to bytes
    const packedBytes = replay.pack();
}
```

### Node.js

```javascript
const { WasmReplay, parse_replay_data_wasm, version } = require('./pkg/rosu_replay.js');
const fs = require('fs');

// Read replay file
const replayData = fs.readFileSync('replay.osr');
const replay = new WasmReplay(replayData);

console.log(`Parsed with rosu-replay v${version()}`);
console.log(`Player: ${replay.username}, Score: ${replay.score}`);
```

## 🎮 Game Mode Support

This library supports all osu! game modes with mode-specific event data:

| Mode | Enum | Event Type | Data Fields |
|------|------|------------|-------------|
| osu!standard | `GameMode::Std` | `ReplayEventOsu` | x, y coordinates + key states |
| osu!taiko | `GameMode::Taiko` | `ReplayEventTaiko` | drum position + hit types |
| osu!catch | `GameMode::Catch` | `ReplayEventCatch` | horizontal position + dash state |
| osu!mania | `GameMode::Mania` | `ReplayEventMania` | multi-lane key states |

## 📁 File Format Support

The .osr format is a binary format used by osu! to store replay data. This library handles:

- ✅ All metadata fields (player, score, mods, timestamp, etc.)
- ✅ LZMA-compressed replay event data (via `liblzma` for optimal performance)
- ✅ Uncompressed replay data
- ✅ Life bar data parsing and generation
- ✅ Timestamp conversion (Windows ticks ↔ Unix timestamps)
- ✅ Both 32-bit and 64-bit replay ID formats
- ✅ All osu! client versions and replay format variations

## 🔧 Advanced Usage

### Custom Compression Settings

```rust
use rosu_replay::{Replay, Packer};

let replay = Replay::from_path("input.osr")?;

// Fastest compression (level 1)
let fast_packer = Packer::new().with_preset(1);
let fast_bytes = replay.pack_with(&fast_packer)?;

// Maximum compression (level 9)
let max_packer = Packer::new().with_preset(9);
let small_bytes = replay.pack_with(&max_packer)?;

// Default compression (level 6) - good balance
let default_bytes = replay.pack()?;
```

### Error Handling

```rust
use rosu_replay::{Replay, ReplayError};

match Replay::from_path("maybe_invalid.osr") {
    Ok(replay) => println!("Loaded replay for {}", replay.username),
    Err(ReplayError::Io(e)) => println!("File error: {}", e),
    Err(ReplayError::Parse(e)) => println!("Parse error: {}", e),
    Err(ReplayError::Lzma(e)) => println!("Compression error: {}", e),
    Err(ReplayError::Utf8(e)) => println!("Text encoding error: {}", e),
}
```

### Performance Tips

```rust
// For batch processing, reuse the same Packer
let packer = Packer::new().with_preset(6);
for replay_path in replay_files {
    let replay = Replay::from_path(replay_path)?;
    let bytes = replay.pack_with(&packer)?; // Faster than creating new Packer each time
    // Process bytes...
}

// Use uncompressed format for faster repeated access
let replay = Replay::from_path("replay.osr")?;
let uncompressed_bytes = replay.pack_uncompressed()?; // Faster to parse later
```

## 📊 Performance

rosu-replay is designed for high performance:

- **Zero-copy parsing** where possible
- **Optimized LZMA compression** via `liblzma` (faster than previous `lzma-rs`)
- **Efficient memory usage** with streaming decompression
- **WASM-optimized** builds for web performance

Benchmarks on a typical replay file:
- **Parse**: ~1-2ms
- **Pack (compressed)**: ~3-5ms  
- **Pack (uncompressed)**: ~0.5ms

## 🧪 Examples

Check out the [`examples/`](examples/) directory for more comprehensive usage:

```bash
# Run the basic example
cargo run --example example_1

# Generate documentation with examples
cargo doc --open

# Run tests including WASM features
cargo test --features wasm
```

## 🔄 Migration from 0.1.0

If you're upgrading from an earlier version:

- ✅ **API is backward compatible** - no breaking changes
- ✅ **Improved performance** with `liblzma` instead of `lzma-rs`
- ✅ **New WASM support** - opt-in with `wasm` feature
- ✅ **Better error handling** with more specific error types

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development

```bash
# Run all tests
cargo test

# Run with WASM features
cargo test --features wasm

# Build documentation
cargo doc --open

# Format code
cargo fmt

# Run linter
cargo clippy
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[kszlim](https://github.com/kszlim)** and contributors for the original [`osrparse`](https://github.com/kszlim/osu-replay-parser) Python library
- The **osu! community** for documenting the .osr file format
- **[ppy](https://github.com/ppy)** for creating osu! and maintaining the replay format
- The **Rust community** for excellent crates like `liblzma`, `wasm-bindgen`, and `chrono`

---

**Made with ❤️ for the osu! community**