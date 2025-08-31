use rosu_replay::{parse_replay_data, GameMode};
use base64::{Engine as _, engine::general_purpose};

/// Test parsing replay data from base64 encoded format (like from osu! API)
#[test]
fn test_parse_api_base64_data() -> Result<(), Box<dyn std::error::Error>> {
    // Create some test replay data and encode it
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2,48|400.0|250.0|0";
    
    // Compress and encode like the API would
    let _compressed = lzma_rs::lzma_compress(&mut replay_data.as_bytes(), &mut Vec::new())
        .map_err(|e| format!("Compression failed: {}", e))?;
    let mut compressed_data = Vec::new();
    lzma_rs::lzma_compress(&mut replay_data.as_bytes(), &mut compressed_data)
        .map_err(|e| format!("Compression failed: {}", e))?;
    
    let base64_data = general_purpose::STANDARD.encode(&compressed_data);
    
    // Parse it like we would from the API
    let events = parse_replay_data(base64_data.as_bytes(), false, false, GameMode::Std)?;
    
    assert_eq!(events.len(), 3);
    
    Ok(())
}

/// Test parsing already decoded data
#[test]
fn test_parse_decoded_data() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2";
    let mut compressed_data = Vec::new();
    lzma_rs::lzma_compress(&mut replay_data.as_bytes(), &mut compressed_data)
        .map_err(|e| format!("Compression failed: {}", e))?;
    
    // Parse with decoded=true (skip base64 decoding)
    let events = parse_replay_data(&compressed_data, true, false, GameMode::Std)?;
    
    assert_eq!(events.len(), 2);
    
    Ok(())
}

/// Test parsing already decompressed data
#[test]
fn test_parse_decompressed_data() -> Result<(), Box<dyn std::error::Error>> {
    let replay_data = "16|256.0|192.0|1,32|300.0|200.0|2,48|400.0|250.0|0";
    
    // Parse with decompressed=true (skip both base64 and lzma)
    let events = parse_replay_data(replay_data.as_bytes(), false, true, GameMode::Std)?;
    
    assert_eq!(events.len(), 3);
    
    Ok(())
}

/// Test parsing different game modes from API data
#[test]
fn test_parse_api_different_modes() -> Result<(), Box<dyn std::error::Error>> {
    // Test taiko data
    let taiko_data = "16|320|0|1,32|640|0|4";
    let events = parse_replay_data(taiko_data.as_bytes(), false, true, GameMode::Taiko)?;
    assert_eq!(events.len(), 2);
    
    // Test catch data  
    let catch_data = "16|256.5|0|1,32|300.0|0|0";
    let events = parse_replay_data(catch_data.as_bytes(), false, true, GameMode::Catch)?;
    assert_eq!(events.len(), 2);
    
    // Test mania data
    let mania_data = "16|5|0|0,32|10|0|0";
    let events = parse_replay_data(mania_data.as_bytes(), false, true, GameMode::Mania)?;
    assert_eq!(events.len(), 2);
    
    Ok(())
}

/// Test error handling for invalid base64
#[test]
fn test_invalid_base64() {
    let invalid_base64 = b"this is not valid base64!!!";
    let result = parse_replay_data(invalid_base64, false, false, GameMode::Std);
    assert!(result.is_err());
}

/// Test error handling for invalid compressed data
#[test]
fn test_invalid_compressed_data() {
    let invalid_data = vec![0xFF, 0xFE, 0xFD, 0xFC]; // Not valid LZMA data
    let result = parse_replay_data(&invalid_data, true, false, GameMode::Std);
    assert!(result.is_err());
}

/// Test empty API response
#[test]
fn test_empty_api_response() -> Result<(), Box<dyn std::error::Error>> {
    let empty_data = "";
    let events = parse_replay_data(empty_data.as_bytes(), false, true, GameMode::Std)?;
    assert_eq!(events.len(), 0);
    
    Ok(())
}

/// Test large replay data performance
#[test]
fn test_large_replay_data_performance() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a large replay data string
    let mut replay_data = String::new();
    for i in 0..10000 {
        replay_data.push_str(&format!("{}|{}.0|{}.0|1,", i * 16, i % 512, (i * 2) % 384));
    }
    
    let start = std::time::Instant::now();
    let events = parse_replay_data(replay_data.as_bytes(), false, true, GameMode::Std)?;
    let duration = start.elapsed();
    
    assert_eq!(events.len(), 10000);
    // Should complete in reasonable time (< 100ms for this size)
    assert!(duration.as_millis() < 100, "Parsing took too long: {:?}", duration);
    
    Ok(())
}

/// Test concurrent parsing
#[test] 
fn test_concurrent_parsing() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread;
    use std::sync::Arc;
    
    let replay_data = Arc::new("16|256.0|192.0|1,32|300.0|200.0|2,48|400.0|250.0|0".to_string());
    
    let handles: Vec<_> = (0..4).map(|_| {
        let data = Arc::clone(&replay_data);
        thread::spawn(move || {
            parse_replay_data(data.as_bytes(), false, true, GameMode::Std)
        })
    }).collect();
    
    for handle in handles {
        let events = handle.join().unwrap()?;
        assert_eq!(events.len(), 3);
    }
    
    Ok(())
}
