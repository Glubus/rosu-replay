//! WebAssembly bindings for rosu-replay
//!
//! This module provides JavaScript-compatible bindings for the rosu-replay library,
//! allowing it to be used in web browsers and Node.js environments.

use crate::{error::ReplayError, replay::Replay, GameMode};
use wasm_bindgen::prelude::*;

// Import the `console.log` function from the `console` module for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Utility macro for console logging (currently unused but available)
#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WASM-compatible wrapper for ReplayError
#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmReplayError {
    inner: String,
}

impl std::fmt::Display for WasmReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for WasmReplayError {}

impl From<ReplayError> for WasmReplayError {
    fn from(error: ReplayError) -> Self {
        WasmReplayError {
            inner: error.to_string(),
        }
    }
}

#[wasm_bindgen]
impl WasmReplayError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.inner.clone()
    }
}

/// WASM-compatible wrapper for GameMode
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum WasmGameMode {
    Std = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

impl From<WasmGameMode> for GameMode {
    fn from(mode: WasmGameMode) -> Self {
        match mode {
            WasmGameMode::Std => GameMode::Std,
            WasmGameMode::Taiko => GameMode::Taiko,
            WasmGameMode::Catch => GameMode::Catch,
            WasmGameMode::Mania => GameMode::Mania,
        }
    }
}

impl From<GameMode> for WasmGameMode {
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Std => WasmGameMode::Std,
            GameMode::Taiko => WasmGameMode::Taiko,
            GameMode::Catch => WasmGameMode::Catch,
            GameMode::Mania => WasmGameMode::Mania,
        }
    }
}

/// WASM-compatible wrapper for Replay
#[wasm_bindgen]
pub struct WasmReplay {
    inner: Replay,
}

#[wasm_bindgen]
impl WasmReplay {
    /// Parse a replay from bytes
    #[wasm_bindgen(constructor)]
    pub fn from_bytes(data: &[u8]) -> Result<WasmReplay, WasmReplayError> {
        let replay = Replay::from_bytes(data)?;
        Ok(WasmReplay { inner: replay })
    }

    /// Get the player username
    #[wasm_bindgen(getter)]
    pub fn username(&self) -> String {
        self.inner.username.clone()
    }

    /// Get the beatmap MD5 hash
    #[wasm_bindgen(getter)]
    pub fn beatmap_hash(&self) -> String {
        self.inner.beatmap_hash.clone()
    }

    /// Get the replay hash
    #[wasm_bindgen(getter)]
    pub fn replay_hash(&self) -> String {
        self.inner.replay_hash.clone()
    }

    /// Get the score
    #[wasm_bindgen(getter)]
    pub fn score(&self) -> u32 {
        self.inner.score
    }

    /// Get the max combo
    #[wasm_bindgen(getter)]
    pub fn max_combo(&self) -> u16 {
        self.inner.max_combo
    }

    /// Get the number of 300s
    #[wasm_bindgen(getter)]
    pub fn count_300(&self) -> u16 {
        self.inner.count_300
    }

    /// Get the number of 100s
    #[wasm_bindgen(getter)]
    pub fn count_100(&self) -> u16 {
        self.inner.count_100
    }

    /// Get the number of 50s
    #[wasm_bindgen(getter)]
    pub fn count_50(&self) -> u16 {
        self.inner.count_50
    }

    /// Get the number of gekis
    #[wasm_bindgen(getter)]
    pub fn count_geki(&self) -> u16 {
        self.inner.count_geki
    }

    /// Get the number of katus
    #[wasm_bindgen(getter)]
    pub fn count_katu(&self) -> u16 {
        self.inner.count_katu
    }

    /// Get the number of misses
    #[wasm_bindgen(getter)]
    pub fn count_miss(&self) -> u16 {
        self.inner.count_miss
    }

    /// Get the game mode
    #[wasm_bindgen(getter)]
    pub fn mode(&self) -> WasmGameMode {
        self.inner.mode.into()
    }

    /// Check if the replay is perfect (no misses)
    #[wasm_bindgen(getter)]
    pub fn is_perfect(&self) -> bool {
        self.inner.count_miss == 0
    }

    /// Get the number of replay events
    #[wasm_bindgen(getter)]
    pub fn event_count(&self) -> usize {
        self.inner.replay_data.len()
    }

    /// Pack the replay back to bytes
    pub fn pack(&self) -> Result<Vec<u8>, WasmReplayError> {
        Ok(self.inner.pack()?)
    }

    /// Pack the replay without compression
    pub fn pack_uncompressed(&self) -> Result<Vec<u8>, WasmReplayError> {
        Ok(self.inner.pack_uncompressed()?)
    }
}

/// Parse replay data from bytes (like from osu! API)
#[wasm_bindgen]
pub fn parse_replay_data_wasm(
    data: &[u8],
    decoded: bool,
    decompressed: bool,
    mode: WasmGameMode,
) -> Result<usize, WasmReplayError> {
    let events = crate::parse_replay_data(data, decoded, decompressed, mode.into())?;
    Ok(events.len())
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get library name
#[wasm_bindgen]
pub fn name() -> String {
    env!("CARGO_PKG_NAME").to_string()
}
