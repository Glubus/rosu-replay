//! Error types for replay parsing and writing operations.

use thiserror::Error;

/// Errors that can occur when parsing or writing replay files.
#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("LZMA decompression error: {0}")]
    Lzma(String),
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    
    #[error("String parsing error: {0}")]
    Parse(String),
    
    #[error("Invalid replay format: {0}")]
    InvalidFormat(String),
    
    #[error("Unexpected end of data")]
    UnexpectedEof,
    
    #[error("Invalid string byte: expected 0x00 or 0x0b, got {0:#x}")]
    InvalidStringByte(u8),
}
