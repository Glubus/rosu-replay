#![allow(dead_code)]
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use liblzma::{
    stream::{LzmaOptions, Stream},
    write::XzEncoder,
};
use serde_json::{json, Value};
use std::io::{Cursor, Read, Write};

pub fn compress(data: &[u8]) -> Vec<u8> {
    let stream = Stream::new_lzma_encoder(&LzmaOptions::new_preset(1).unwrap()).unwrap();
    let mut encoder = XzEncoder::new_stream(Vec::new(), stream);
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

pub fn score() -> Value {
    json!({"client_version":"2026.901", "online_id":-1, "rank":"A",
        "mods":[{"acronym":"DT","settings":{"speed_change":1.2}}],
        "statistics":{"great":12}, "maximum_statistics":{"great":15}})
}

// Header built independently of the production reader/writer. Empty hash/name
// fields keep the replay-block length offset fixed at 40 bytes.
pub fn fixture(mode: u8, version: u32, id: i64, frames: &str, tail: &[u8]) -> Vec<u8> {
    let mut out = vec![mode];
    out.write_u32::<LittleEndian>(version).unwrap();
    out.extend([0; 3]);
    for count in [12, 2, 1, 3, 4, 5] {
        out.write_u16::<LittleEndian>(count).unwrap();
    }
    out.write_u32::<LittleEndian>(123456).unwrap();
    out.write_u16::<LittleEndian>(42).unwrap();
    out.push(0);
    out.write_u32::<LittleEndian>(0).unwrap();
    out.push(0);
    out.write_i64::<LittleEndian>(638000000001234567).unwrap();
    let block = compress(frames.as_bytes());
    out.write_i32::<LittleEndian>(block.len() as i32).unwrap();
    out.extend(block);
    if version >= 20140721 {
        out.write_i64::<LittleEndian>(id).unwrap();
    } else if version >= 20121008 {
        out.write_i32::<LittleEndian>(id as i32).unwrap();
    }
    out.extend(tail);
    out
}

pub fn json_tail(value: &Value) -> Vec<u8> {
    let block = compress(&serde_json::to_vec(value).unwrap());
    let mut out = (block.len() as i32).to_le_bytes().to_vec();
    out.extend(block);
    out
}

pub fn suffix(bytes: &[u8]) -> &[u8] {
    let mut reader = Cursor::new(bytes);
    reader.set_position(5);
    for _ in 0..3 {
        skip_string(&mut reader);
    }
    reader.set_position(reader.position() + 23);
    skip_string(&mut reader);
    reader.set_position(reader.position() + 8);
    let size = reader.read_i32::<LittleEndian>().unwrap();
    reader.set_position(reader.position() + size as u64);
    &bytes[reader.position() as usize..]
}

fn skip_string(reader: &mut Cursor<&[u8]>) {
    if reader.read_u8().unwrap() == 0 {
        return;
    }
    let mut len = 0;
    let mut shift = 0;
    loop {
        let b = reader.read_u8().unwrap();
        len |= u64::from(b & 127) << shift;
        if b < 128 {
            break;
        }
        shift += 7;
    }
    reader.set_position(reader.position() + len);
}

pub fn read_json(bytes: &[u8]) -> Value {
    let tail = suffix(bytes);
    let size = i32::from_le_bytes(tail[8..12].try_into().unwrap()) as usize;
    assert_eq!(tail.len(), 12 + size, "unexpected trailing data");
    let mut decoded = Vec::new();
    liblzma::read::XzDecoder::new_multi_decoder(&tail[12..])
        .read_to_end(&mut decoded)
        .unwrap();
    serde_json::from_slice(&decoded).unwrap()
}
