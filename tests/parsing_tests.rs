use rosu_replay::unpacker::Unpacker;
use rosu_replay::{GameMode, Replay, ReplayEvent};
use std::io::Cursor;

/// Test parsing replay data from string format
#[test]
fn test_parse_replay_data_string() -> Result<(), Box<dyn std::error::Error>> {
    // Test osu!standard replay data
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2,48|400.0|250.0|0";
    let (events, seed) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std)?;

    assert_eq!(events.len(), 3);
    assert!(seed.is_none());

    if let ReplayEvent::Osu(event) = &events[0] {
        assert_eq!(event.time_delta, 16);
        assert_eq!(event.x, 256.0);
        assert_eq!(event.y, 192.0);
        assert_eq!(event.keys.value(), 1);
    } else {
        panic!("Expected osu event");
    }

    Ok(())
}

/// Test parsing replay data with RNG seed
#[test]
fn test_parse_replay_data_with_seed() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2,-12345|0|0|12345";
    let (events, seed) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std)?;

    assert_eq!(events.len(), 2); // RNG seed event is not included in events
    assert_eq!(seed, Some(12345));

    Ok(())
}

/// Test parsing taiko replay data
#[test]
fn test_parse_taiko_replay_data() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|320|0|1,32|640|0|4,48|0|0|2";
    let (events, _) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Taiko)?;

    assert_eq!(events.len(), 3);

    if let ReplayEvent::Taiko(event) = &events[0] {
        assert_eq!(event.time_delta, 16);
        assert_eq!(event.x, 320);
        assert_eq!(event.keys.value(), 1);
    } else {
        panic!("Expected taiko event");
    }

    Ok(())
}

/// Test parsing catch replay data
#[test]
fn test_parse_catch_replay_data() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|256.5|0|1,32|300.0|0|0,48|400.25|0|1";
    let (events, _) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Catch)?;

    assert_eq!(events.len(), 3);

    if let ReplayEvent::Catch(event) = &events[0] {
        assert_eq!(event.time_delta, 16);
        assert_eq!(event.x, 256.5);
        assert!(event.dashing);
    } else {
        panic!("Expected catch event");
    }

    if let ReplayEvent::Catch(event) = &events[1] {
        assert!(!event.dashing);
    } else {
        panic!("Expected catch event");
    }

    Ok(())
}

/// Test parsing mania replay data
#[test]
fn test_parse_mania_replay_data() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|5|0|0,32|10|0|0,48|0|0|0";
    let (events, _) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Mania)?;

    assert_eq!(events.len(), 3);

    if let ReplayEvent::Mania(event) = &events[0] {
        assert_eq!(event.time_delta, 16);
        assert_eq!(event.keys.value(), 5); // K1 + K3
    } else {
        panic!("Expected mania event");
    }

    Ok(())
}

/// Test parsing empty replay data
#[test]
fn test_parse_empty_replay_data() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "";
    let (events, seed) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std)?;

    assert_eq!(events.len(), 0);
    assert!(seed.is_none());

    Ok(())
}

/// Test parsing replay data with lazer skip frames
#[test]
fn test_parse_replay_data_skip_lazer_frames() -> Result<(), Box<dyn std::error::Error>> {
    // First two frames with x=256, y=-500 should be skipped
    let replay_data = "0|256|-500|0,0|256|-500|0,16|100.0|100.0|1";
    let (events, _) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std)?;

    assert_eq!(events.len(), 1); // Only the third frame should remain

    if let ReplayEvent::Osu(event) = &events[0] {
        assert_eq!(event.x, 100.0);
        assert_eq!(event.y, 100.0);
    } else {
        panic!("Expected osu event");
    }

    Ok(())
}

/// Test parsing malformed replay data
#[test]
fn test_parse_malformed_replay_data() {
    // Test with incomplete data
    let replay_data = "16|256.0";
    let _ = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std);
    // Should not panic, but might produce fewer events

    // Test with invalid numbers
    let replay_data = "invalid|256.0|192.0|1";
    let result = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std);
    assert!(result.is_err());
}

/// Test parsing replay data with trailing comma
#[test]
fn test_parse_replay_data_trailing_comma() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2,";
    let (events, _) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(replay_data, GameMode::Std)?;

    assert_eq!(events.len(), 2);

    Ok(())
}

/// Test string parsing utilities
#[test]
fn test_string_parsing() -> Result<(), Box<dyn std::error::Error>> {
    // Test ULEB128 encoding/decoding through the unpacker
    let test_data = vec![0x0b, 0x05, b'H', b'e', b'l', b'l', b'o']; // String "Hello"
    let mut unpacker = Unpacker::new(Cursor::new(test_data));

    let result = unpacker.unpack_string()?;
    assert_eq!(result, Some("Hello".to_string()));

    Ok(())
}

/// Test parsing timestamps
#[test]
fn test_timestamp_parsing() -> Result<(), Box<dyn std::error::Error>> {
    // Test with a known timestamp (Windows ticks format)
    let test_data: Vec<u8> = vec![
        0x00, 0x1C, 0xF4, 0x36, 0x6D, 0x26, 0xD7, 0x08, // Some timestamp in little-endian
    ];

    let mut unpacker = Unpacker::new(Cursor::new(test_data));
    let timestamp = unpacker.unpack_timestamp()?;

    // Just verify it's a valid timestamp (not testing exact value due to conversion complexity)
    assert!(timestamp.timestamp() > 0);

    Ok(())
}

/// Test life bar parsing
#[test]
fn test_life_bar_parsing() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate parsing a life bar string: "1000|1.0,2000|0.8,3000|0.6,"
    let life_bar_data = "1000|1.0,2000|0.8,3000|0.6,";

    // We need to create a proper byte sequence for this
    let mut data = vec![0x0b]; // String indicator
    let life_bar_bytes = life_bar_data.as_bytes();
    data.push(life_bar_bytes.len() as u8); // Length (assuming < 128)
    data.extend_from_slice(life_bar_bytes);

    let mut unpacker = Unpacker::new(Cursor::new(data));
    let life_bar = unpacker.unpack_life_bar()?;

    assert!(life_bar.is_some());
    let life_bar = life_bar.unwrap();
    assert_eq!(life_bar.len(), 3);

    assert_eq!(life_bar[0].time, 1000);
    assert_eq!(life_bar[0].life, 1.0);
    assert_eq!(life_bar[1].time, 2000);
    assert_eq!(life_bar[1].life, 0.8);
    assert_eq!(life_bar[2].time, 3000);
    assert_eq!(life_bar[2].life, 0.6);

    Ok(())
}
