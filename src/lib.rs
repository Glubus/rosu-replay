//! # rosu-replay
//!
//! A Rust library for parsing and writing osu! replay files (.osr format).
//!
//! This library is a faithful port of the Python [`osrparse`](https://github.com/kszlim/osu-replay-parser) library,
//! providing the same functionality for parsing and manipulating osu! replay files in Rust.
//!
//! ## Features
//!
//! - **Parse .osr replay files** - Read replay files from disk or memory
//! - **Extract replay data and metadata** - Access all replay information including:
//!   - Player information (username, score, combo, etc.)
//!   - Game metadata (mode, mods, timestamp, etc.)
//!   - Hit statistics (300s, 100s, 50s, misses, etc.)
//!   - Replay events (cursor movement, key presses)
//!   - Life bar data over time
//! - **Write replay files** - Save modified replays back to .osr format
//! - **Support all game modes** - osu!standard, osu!taiko, osu!catch, osu!mania
//! - **API compatibility** - Parse replay data from osu! API v1 responses
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rosu-replay = "0.1"
//! ```
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use rosu_replay::Replay;
//!
//! // Parse a replay file
//! let replay = Replay::from_path("path/to/replay.osr")?;
//!
//! println!("Player: {}", replay.username);
//! println!("Score: {}", replay.score);
//! println!("Max Combo: {}", replay.max_combo);
//! println!("Game Mode: {:?}", replay.mode);
//!
//! // Access replay events
//! for event in &replay.replay_data {
//!     match event {
//!         rosu_replay::ReplayEvent::Osu(osu_event) => {
//!             println!("Cursor at ({}, {}) at time +{}ms",
//!                 osu_event.x, osu_event.y, osu_event.time_delta);
//!         }
//!         _ => {} // Handle other game modes
//!     }
//! }
//!
//! // Write the replay back
//! replay.write_path("output.osr")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Working with API Data
//!
//! ```rust,no_run
//! use rosu_replay::{parse_replay_data, GameMode};
//!
//! // Parse replay data from osu! API v1
//! let api_data = b"base64_encoded_replay_data";
//! let events = parse_replay_data(api_data, false, false, GameMode::Std)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Attribution
//!
//! This library is a port of the Python [`osrparse`](https://github.com/kszlim/osu-replay-parser) library
//! by kszlim and contributors. The original Python implementation provided the foundation for understanding
//! the .osr file format and replay data structures.
//!
//! ## Examples
//!
//! See the `examples/` directory for more comprehensive usage examples.

pub mod error;
pub mod packer;
pub mod replay;
pub mod types;
pub mod unpacker;

pub use error::ReplayError;
pub use replay::Replay;
pub use types::*;

/// Parse replay data from a string (for API usage)
pub fn parse_replay_data(
    data_string: &[u8],
    decoded: bool,
    decompressed: bool,
    mode: GameMode,
) -> Result<Vec<ReplayEvent>, ReplayError> {
    replay::parse_replay_data(data_string, decoded, decompressed, mode)
}
