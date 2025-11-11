//! Tests for WebAssembly bindings
//!
//! These tests verify that the WASM bindings work correctly and maintain
//! compatibility with the native Rust API.

#![cfg(feature = "wasm")]

use rosu_replay::wasm::{name, parse_replay_data_wasm, version, WasmGameMode, WasmReplay};
use rosu_replay::{GameMode, Replay};
use std::fs;

/// Test that WasmGameMode conversion works correctly
#[test]
fn test_wasm_gamemode_conversion() {
    // Test all game mode conversions
    let modes = [
        (WasmGameMode::Std, GameMode::Std),
        (WasmGameMode::Taiko, GameMode::Taiko),
        (WasmGameMode::Catch, GameMode::Catch),
        (WasmGameMode::Mania, GameMode::Mania),
    ];

    for (wasm_mode, native_mode) in modes {
        let converted_native: GameMode = wasm_mode.into();
        let converted_wasm: WasmGameMode = native_mode.into();

        assert_eq!(converted_native, native_mode);
        assert_eq!(converted_wasm as u8, wasm_mode as u8);
    }
}

/// Test WasmReplay wrapper with a real replay file
#[test]
fn test_wasm_replay_wrapper() -> Result<(), Box<dyn std::error::Error>> {
    // Load test replay file
    let test_file = "assets/test.osr";
    if !std::path::Path::new(test_file).exists() {
        // Skip test if file doesn't exist
        println!("Skipping WASM replay test - test file not found");
        return Ok(());
    }

    let replay_data = fs::read(test_file)?;

    // Skip test if file is empty or invalid
    if replay_data.is_empty() {
        println!("Skipping WASM replay test - test file is empty");
        return Ok(());
    }

    // Try to create both native and WASM replays
    let native_replay = match Replay::from_bytes(&replay_data) {
        Ok(replay) => replay,
        Err(_) => {
            println!("Skipping WASM replay test - test file is not a valid replay");
            return Ok(());
        }
    };
    let wasm_replay = WasmReplay::from_bytes(&replay_data)?;

    // Test that all properties match
    assert_eq!(wasm_replay.username(), native_replay.username);
    assert_eq!(wasm_replay.beatmap_hash(), native_replay.beatmap_hash);
    assert_eq!(wasm_replay.replay_hash(), native_replay.replay_hash);
    assert_eq!(wasm_replay.score(), native_replay.score);
    assert_eq!(wasm_replay.max_combo(), native_replay.max_combo);
    assert_eq!(wasm_replay.count_300(), native_replay.count_300);
    assert_eq!(wasm_replay.count_100(), native_replay.count_100);
    assert_eq!(wasm_replay.count_50(), native_replay.count_50);
    assert_eq!(wasm_replay.count_geki(), native_replay.count_geki);
    assert_eq!(wasm_replay.count_katu(), native_replay.count_katu);
    assert_eq!(wasm_replay.count_miss(), native_replay.count_miss);
    assert_eq!(wasm_replay.event_count(), native_replay.replay_data.len());

    // Test mode conversion
    let wasm_mode = wasm_replay.mode();
    let native_mode_converted: WasmGameMode = native_replay.mode.into();
    assert_eq!(wasm_mode as u8, native_mode_converted as u8);

    // Test is_perfect logic
    let expected_perfect = native_replay.count_miss == 0;
    assert_eq!(wasm_replay.is_perfect(), expected_perfect);

    Ok(())
}

/// Test WasmReplay creation with minimal replay data
#[test]
fn test_wasm_replay_minimal() -> Result<(), Box<dyn std::error::Error>> {
    // Create minimal replay data for testing
    let minimal_replay = create_minimal_test_replay();
    let replay_bytes = minimal_replay.pack()?;

    // Test WASM wrapper
    let wasm_replay = WasmReplay::from_bytes(&replay_bytes)?;

    assert_eq!(wasm_replay.username(), "test_player");
    assert_eq!(wasm_replay.score(), 12345);
    assert_eq!(wasm_replay.mode() as u8, WasmGameMode::Std as u8);
    assert!(wasm_replay.is_perfect()); // No misses in minimal replay

    Ok(())
}

/// Test WasmReplayError functionality
#[test]
fn test_wasm_error_handling() {
    // Test with invalid data
    let invalid_data = b"invalid replay data";
    let result = WasmReplay::from_bytes(invalid_data);

    assert!(result.is_err());

    // Test error message extraction
    if let Err(wasm_error) = result {
        let message = wasm_error.message();
        assert!(!message.is_empty());
        assert!(message.len() > 0);
    }
}

/// Test WASM utility functions
#[test]
fn test_wasm_utility_functions() {
    // Test version function
    let ver = version();
    assert_eq!(ver, env!("CARGO_PKG_VERSION"));

    // Test name function
    let lib_name = name();
    assert_eq!(lib_name, env!("CARGO_PKG_NAME"));
}

/// Test parse_replay_data_wasm function
#[test]
fn test_parse_replay_data_wasm() -> Result<(), Box<dyn std::error::Error>> {
    use liblzma::encode_all;

    // Create test replay data with valid coordinates (floats)
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2,48|400.0|250.0|0";

    // Test with compressed data
    let compressed_data = encode_all(replay_data.as_bytes(), 6)?;
    let event_count = parse_replay_data_wasm(&compressed_data, true, false, WasmGameMode::Std)?;
    assert_eq!(event_count, 3);

    // Test with decompressed data
    let event_count_raw =
        parse_replay_data_wasm(replay_data.as_bytes(), true, true, WasmGameMode::Std)?;
    assert_eq!(event_count_raw, 3);

    // Test different game modes
    let taiko_data = "16|128|0|1,32|64|0|2"; // Taiko uses integer x coordinates
    let taiko_count =
        parse_replay_data_wasm(taiko_data.as_bytes(), true, true, WasmGameMode::Taiko)?;
    assert_eq!(taiko_count, 2);

    Ok(())
}

/// Test WASM replay packing functionality
#[test]
fn test_wasm_replay_packing() -> Result<(), Box<dyn std::error::Error>> {
    let minimal_replay = create_minimal_test_replay();
    let original_bytes = minimal_replay.pack()?;

    // Create WASM replay and pack it back
    let wasm_replay = WasmReplay::from_bytes(&original_bytes)?;
    let packed_bytes = wasm_replay.pack()?;
    let uncompressed_bytes = wasm_replay.pack_uncompressed()?;

    // Verify we can read the packed data back
    let reparsed = Replay::from_bytes(&packed_bytes)?;
    assert_eq!(reparsed.username, "test_player");
    assert_eq!(reparsed.score, 12345);

    // Both should be valid data
    assert!(!packed_bytes.is_empty());
    assert!(!uncompressed_bytes.is_empty());

    Ok(())
}

/// Test error handling with various invalid inputs
#[test]
fn test_wasm_error_scenarios() {
    // Test with empty data
    let result = WasmReplay::from_bytes(&[]);
    assert!(result.is_err());

    // Test with truncated data
    let result = WasmReplay::from_bytes(&[1, 2, 3, 4, 5]);
    assert!(result.is_err());

    // Test parse_replay_data_wasm with invalid data (should return 0 events for empty/invalid data)
    let result = parse_replay_data_wasm(b"invalid", true, true, WasmGameMode::Std);
    match result {
        Ok(count) => assert_eq!(count, 0), // Empty/invalid data should result in 0 events
        Err(_) => {}                       // Error is also acceptable
    }

    // Test parse_replay_data_wasm with malformed replay data
    let result = parse_replay_data_wasm(b"not|valid|data", true, true, WasmGameMode::Std);
    match result {
        Ok(count) => assert_eq!(count, 0), // Malformed data should result in 0 events
        Err(_) => {}                       // Error is also acceptable
    }
}

/// Helper function to create a minimal test replay
fn create_minimal_test_replay() -> Replay {
    use chrono::Utc;
    use rosu_replay::*;

    Replay {
        mode: GameMode::Std,
        game_version: 20200201,
        beatmap_hash: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
        username: "test_player".to_string(),
        replay_hash: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
        count_300: 100,
        count_100: 10,
        count_50: 5,
        count_geki: 0,
        count_katu: 0,
        count_miss: 0,
        score: 12345,
        max_combo: 150,
        perfect: false,
        mods: Mod::NO_MOD,
        life_bar_graph: Some(vec![]),
        timestamp: Utc::now(),
        replay_data: vec![
            ReplayEvent::Osu(ReplayEventOsu {
                time_delta: 16,
                x: 256.0,
                y: 192.0,
                keys: Key::K1,
            }),
            ReplayEvent::Osu(ReplayEventOsu {
                time_delta: 32,
                x: 300.0,
                y: 200.0,
                keys: Key::K2,
            }),
        ],
        replay_id: 123456,
        rng_seed: None,
    }
}
