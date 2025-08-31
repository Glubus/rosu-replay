# Test Suite

This directory contains comprehensive tests for the rosu-replay library. The tests are organized into different categories to ensure complete coverage of the library's functionality.

## Test Files

### `integration_tests.rs`
- **Basic data structure tests**: GameMode conversion, Mod operations, Key values
- **Replay creation and roundtrip testing**: Creating replays and verifying pack/unpack works correctly
- **Game mode event testing**: Testing all four game modes (standard, taiko, catch, mania)
- **Life bar data testing**: Verifying life bar state parsing
- **Time calculation testing**: Ensuring replay timing calculations are correct

### `parsing_tests.rs`
- **String format parsing**: Testing replay data parsing from string format
- **RNG seed handling**: Verifying RNG seed extraction from replay data
- **Multi-mode parsing**: Testing parsing for all game modes
- **Edge case handling**: Empty data, trailing commas, lazer skip frames
- **Malformed data handling**: Testing graceful handling of invalid input
- **String utilities**: ULEB128 encoding, timestamp parsing, life bar parsing

### `api_tests.rs`
- **Base64 decoding**: Testing API data format (base64 encoded)
- **Compression handling**: LZMA decompression of replay data
- **Multi-format support**: decoded, decompressed, and raw data
- **Performance testing**: Large replay data handling
- **Concurrent access**: Thread-safe parsing
- **Error handling**: Invalid base64, invalid compressed data

### `error_tests.rs`
- **Empty data errors**: Testing behavior with no input data
- **Invalid format errors**: Testing various invalid file formats
- **Truncated file errors**: Testing incomplete data handling
- **String encoding errors**: Invalid UTF-8, invalid string bytes
- **Compression errors**: LZMA decompression failures
- **Error display**: Verifying error messages are helpful
- **Error chaining**: Testing error source chains
- **Concurrent error handling**: Thread-safe error handling

## Test Coverage

The test suite covers:

- ✅ **Parsing functionality** - All game modes, various data formats
- ✅ **Writing functionality** - Roundtrip testing (parse → modify → write → parse)
- ✅ **Error handling** - All error types with appropriate messages
- ✅ **Performance** - Large data sets, concurrent access
- ✅ **API compatibility** - Base64/LZMA data from osu! API
- ✅ **Edge cases** - Empty data, malformed input, truncated files
- ✅ **Thread safety** - Concurrent parsing and error handling

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test integration_tests
cargo test --test parsing_tests
cargo test --test api_tests
cargo test --test error_tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_replay_roundtrip

# Run tests with release optimizations
cargo test --release
```

## Test Statistics

Current test count: **42 tests total**
- Integration tests: 11 tests
- Parsing tests: 12 tests  
- API tests: 9 tests
- Error tests: 10 tests

All tests pass with 0 failures, providing confidence in the library's reliability and correctness.
