use crate::ReplayError;
use liblzma::{
    read::XzDecoder,
    stream::{LzmaOptions, Stream},
    write::XzEncoder,
};
use std::io::{Read, Write};

pub(crate) fn encode(data: &[u8], preset: u32) -> Result<Vec<u8>, ReplayError> {
    let stream = Stream::new_lzma_encoder(&LzmaOptions::new_preset(preset)?)?;
    let mut encoder = XzEncoder::new_stream(Vec::new(), stream);
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub(crate) fn decode(data: &[u8], limit: usize) -> Result<Vec<u8>, ReplayError> {
    // Bound the LZMA dictionary independently of the decompressed output limit.
    let stream = Stream::new_auto_decoder(256 * 1024 * 1024, 0)?;
    let mut decoder = XzDecoder::new_stream(data, stream).take((limit as u64).saturating_add(1));
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    if result.len() > limit {
        return Err(ReplayError::InvalidFormat(
            "decompressed block exceeds configured limit".into(),
        ));
    }
    if decoder.get_ref().total_in() != data.len() as u64 {
        return Err(ReplayError::InvalidFormat(
            "trailing bytes in compressed block".into(),
        ));
    }
    Ok(result)
}
