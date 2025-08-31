use rosu_replay::{Replay, ReplayError};
use std::io::Cursor;

/// Test various error conditions
#[test]
fn test_empty_data_error() {
    let result = Replay::from_bytes(&[]);
    assert!(result.is_err());
    
    if let Err(ReplayError::UnexpectedEof) = result {
        // Expected error type
    } else if let Err(ReplayError::Io(_)) = result {
        // Also acceptable as IO error
    } else {
        panic!("Expected UnexpectedEof or IO error, got: {:?}", result);
    }
}

/// Test invalid file format
#[test]
fn test_invalid_format_error() {
    // Create some invalid binary data
    let invalid_data = vec![0xFF; 100];
    let result = Replay::from_bytes(&invalid_data);
    assert!(result.is_err());
}

/// Test truncated file error
#[test]
fn test_truncated_file_error() {
    // Create a file that starts correctly but is truncated
    let mut data = Vec::new();
    data.push(0); // Valid game mode
    data.extend_from_slice(&[1, 0, 0, 0]); // Valid game version
    // But then truncate before the beatmap hash
    
    let result = Replay::from_bytes(&data);
    assert!(result.is_err());
}

/// Test invalid string byte
#[test]
fn test_invalid_string_byte_error() {
    let mut data = Vec::new();
    data.push(0); // Valid game mode
    data.extend_from_slice(&[1, 0, 0, 0]); // Valid game version
    data.push(0xFF); // Invalid string indicator (should be 0x00 or 0x0b)
    
    let result = Replay::from_bytes(&data);
    assert!(result.is_err());
    
    if let Err(ReplayError::InvalidStringByte(byte)) = result {
        assert_eq!(byte, 0xFF);
    } else {
        panic!("Expected InvalidStringByte error, got: {:?}", result);
    }
}

/// Test LZMA decompression error
#[test]
fn test_lzma_error() {
    use rosu_replay::unpacker::Unpacker;
    
    // Create a replay with invalid compressed data
    let mut data = Vec::new();
    // Add valid header
    data.push(0); // game mode
    data.extend_from_slice(&[1, 0, 0, 0]); // game version
    data.push(0x00); // empty beatmap hash
    data.push(0x00); // empty username
    data.push(0x00); // empty replay hash
    data.extend_from_slice(&[0, 0]); // count_300
    data.extend_from_slice(&[0, 0]); // count_100
    data.extend_from_slice(&[0, 0]); // count_50
    data.extend_from_slice(&[0, 0]); // count_geki
    data.extend_from_slice(&[0, 0]); // count_katu
    data.extend_from_slice(&[0, 0]); // count_miss
    data.extend_from_slice(&[0, 0, 0, 0]); // score
    data.extend_from_slice(&[0, 0]); // max_combo
    data.push(0); // perfect
    data.extend_from_slice(&[0, 0, 0, 0]); // mods
    data.push(0x00); // empty life bar
    data.extend_from_slice(&[0; 8]); // timestamp
    
    // Add invalid compressed replay data
    data.extend_from_slice(&[10, 0, 0, 0]); // length = 10
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7, 0xF6]); // invalid LZMA data
    
    let mut unpacker = Unpacker::new(Cursor::new(data));
    let result = unpacker.unpack_play_data(rosu_replay::GameMode::Std);
    assert!(result.is_err());
    
    // The error could be either LZMA or IO (UnexpectedEof) depending on how the unpacker fails
    match result {
        Err(ReplayError::Lzma(_)) | Err(ReplayError::Io(_)) => {
            // Both are acceptable error types for invalid compressed data
        }
        _ => panic!("Expected LZMA or IO error, got: {:?}", result),
    }
}

/// Test UTF-8 conversion error
#[test]
fn test_utf8_error() {
    use rosu_replay::unpacker::Unpacker;
    
    // Create data with invalid UTF-8 in string
    let mut data = Vec::new();
    data.push(0x0b); // String indicator
    data.push(4); // Length = 4
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]); // Invalid UTF-8
    
    let mut unpacker = Unpacker::new(Cursor::new(data));
    let result = unpacker.unpack_string();
    assert!(result.is_err());
    
    if let Err(ReplayError::Utf8(_)) = result {
        // Expected error type
    } else {
        panic!("Expected UTF-8 error, got: {:?}", result);
    }
}

/// Test parse error for replay data
#[test]
fn test_parse_error() {
    use rosu_replay::unpacker::Unpacker;
    
    // Test with invalid number in replay data
    let invalid_replay_data = "invalid_number|256.0|192.0|1";
    let result = Unpacker::<Cursor<&[u8]>>::parse_replay_data(invalid_replay_data, rosu_replay::GameMode::Std);
    assert!(result.is_err());
    
    if let Err(ReplayError::Parse(_)) = result {
        // Expected error type
    } else {
        panic!("Expected Parse error, got: {:?}", result);
    }
}

/// Test error display messages
#[test]
fn test_error_display() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let replay_error = ReplayError::Io(io_error);
    let error_message = format!("{}", replay_error);
    assert!(error_message.contains("IO error"));
    
    let lzma_error = ReplayError::Lzma("Decompression failed".to_string());
    let error_message = format!("{}", lzma_error);
    assert!(error_message.contains("LZMA decompression error"));
    
    let parse_error = ReplayError::Parse("Invalid number".to_string());
    let error_message = format!("{}", parse_error);
    assert!(error_message.contains("String parsing error"));
    
    let format_error = ReplayError::InvalidFormat("Bad header".to_string());
    let error_message = format!("{}", format_error);
    assert!(error_message.contains("Invalid replay format"));
    
    let eof_error = ReplayError::UnexpectedEof;
    let error_message = format!("{}", eof_error);
    assert!(error_message.contains("Unexpected end of data"));
    
    let string_error = ReplayError::InvalidStringByte(0xFF);
    let error_message = format!("{}", string_error);
    assert!(error_message.contains("Invalid string byte"));
    assert!(error_message.contains("0xff"));
}

/// Test error source chain
#[test]
fn test_error_source() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let replay_error = ReplayError::Io(io_error);
    
    // Test that the source is available
    assert!(std::error::Error::source(&replay_error).is_some());
}

/// Test concurrent error handling
#[test]
fn test_concurrent_error_handling() {
    use std::thread;
    
    let handles: Vec<_> = (0..4).map(|_| {
        thread::spawn(|| {
            let invalid_data = vec![0xFF; 10];
            Replay::from_bytes(&invalid_data)
        })
    }).collect();
    
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_err());
    }
}
