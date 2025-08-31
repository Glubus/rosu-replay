use crate::{error::ReplayError, replay::Replay, types::*};
use byteorder::{LittleEndian, WriteBytesExt};
use lzma_rs::lzma_compress;
use std::io::Write;

/// Helper struct for packing data into .osr format
pub struct Packer {
    preset: u32,
}

impl Default for Packer {
    fn default() -> Self {
        Self {
            preset: 6, // Default compression level
        }
    }
}

impl Packer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_preset(mut self, preset: u32) -> Self {
        self.preset = preset;
        self
    }

    fn pack_byte(&self, writer: &mut impl Write, data: u8) -> Result<(), ReplayError> {
        writer.write_u8(data)?;
        Ok(())
    }

    fn pack_short(&self, writer: &mut impl Write, data: u16) -> Result<(), ReplayError> {
        writer.write_u16::<LittleEndian>(data)?;
        Ok(())
    }

    fn pack_int(&self, writer: &mut impl Write, data: u32) -> Result<(), ReplayError> {
        writer.write_u32::<LittleEndian>(data)?;
        Ok(())
    }

    fn pack_long(&self, writer: &mut impl Write, data: i64) -> Result<(), ReplayError> {
        writer.write_i64::<LittleEndian>(data)?;
        Ok(())
    }

    fn pack_uleb128(&self, writer: &mut impl Write, mut value: usize) -> Result<(), ReplayError> {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;

            if value != 0 {
                byte |= 0x80;
            }

            writer.write_u8(byte)?;

            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn pack_string(&self, writer: &mut impl Write, data: Option<&str>) -> Result<(), ReplayError> {
        match data {
            None | Some("") => {
                self.pack_byte(writer, 0x00)?;
            }
            Some(s) => {
                self.pack_byte(writer, 0x0b)?;
                let bytes = s.as_bytes();
                self.pack_uleb128(writer, bytes.len())?;
                writer.write_all(bytes)?;
            }
        }
        Ok(())
    }

    fn pack_timestamp(
        &self,
        writer: &mut impl Write,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ReplayError> {
        // Windows ticks start from year 1 AD, Unix epoch starts from 1970
        // There are 621355968000000000 ticks between year 1 and Unix epoch
        const TICKS_TO_UNIX_EPOCH: i64 = 621355968000000000;
        const TICKS_PER_SECOND: i64 = 10_000_000;

        let unix_timestamp = timestamp.timestamp();
        let nanoseconds = timestamp.timestamp_subsec_nanos();

        let ticks =
            TICKS_TO_UNIX_EPOCH + (unix_timestamp * TICKS_PER_SECOND) + (nanoseconds as i64 / 100);

        self.pack_long(writer, ticks)?;
        Ok(())
    }

    fn pack_life_bar(
        &self,
        writer: &mut impl Write,
        life_bar_graph: &Option<Vec<LifeBarState>>,
    ) -> Result<(), ReplayError> {
        match life_bar_graph {
            None => {
                self.pack_string(writer, None)?;
            }
            Some(states) => {
                let mut data = String::new();
                for state in states {
                    let life = if state.life.fract() == 0.0 {
                        (state.life as i32).to_string()
                    } else {
                        state.life.to_string()
                    };
                    data.push_str(&format!("{}|{},", state.time, life));
                }
                self.pack_string(writer, Some(&data))?;
            }
        }
        Ok(())
    }

    fn pack_replay_data(
        &self,
        writer: &mut impl Write,
        replay_data: &[ReplayEvent],
        rng_seed: Option<i32>,
    ) -> Result<(), ReplayError> {
        let mut data = String::new();

        for event in replay_data {
            match event {
                ReplayEvent::Osu(event) => {
                    data.push_str(&format!(
                        "{}|{}|{}|{},",
                        event.time_delta,
                        event.x,
                        event.y,
                        event.keys.value()
                    ));
                }
                ReplayEvent::Taiko(event) => {
                    data.push_str(&format!(
                        "{}|{}|0|{},",
                        event.time_delta,
                        event.x,
                        event.keys.value()
                    ));
                }
                ReplayEvent::Catch(event) => {
                    data.push_str(&format!(
                        "{}|{}|0|{},",
                        event.time_delta,
                        event.x,
                        if event.dashing { 1 } else { 0 }
                    ));
                }
                ReplayEvent::Mania(event) => {
                    data.push_str(&format!("{}|{}|0|0,", event.time_delta, event.keys.value()));
                }
            }
        }

        if let Some(seed) = rng_seed {
            data.push_str(&format!("-12345|0|0|{},", seed));
        }

        // Compress the data
        let data_bytes = data.as_bytes();
        let mut compressed = Vec::new();
        lzma_compress(&mut &data_bytes[..], &mut compressed)
            .map_err(|e| ReplayError::Lzma(format!("{}", e)))?;

        // Write length and compressed data
        self.pack_int(writer, compressed.len() as u32)?;
        writer.write_all(&compressed)?;

        Ok(())
    }

    pub fn pack(&self, replay: &Replay) -> Result<Vec<u8>, ReplayError> {
        let mut buffer = Vec::new();

        self.pack_byte(&mut buffer, replay.mode as u8)?;
        self.pack_int(&mut buffer, replay.game_version)?;
        self.pack_string(&mut buffer, Some(&replay.beatmap_hash))?;
        self.pack_string(&mut buffer, Some(&replay.username))?;
        self.pack_string(&mut buffer, Some(&replay.replay_hash))?;
        self.pack_short(&mut buffer, replay.count_300)?;
        self.pack_short(&mut buffer, replay.count_100)?;
        self.pack_short(&mut buffer, replay.count_50)?;
        self.pack_short(&mut buffer, replay.count_geki)?;
        self.pack_short(&mut buffer, replay.count_katu)?;
        self.pack_short(&mut buffer, replay.count_miss)?;
        self.pack_int(&mut buffer, replay.score)?;
        self.pack_short(&mut buffer, replay.max_combo)?;
        self.pack_byte(&mut buffer, if replay.perfect { 1 } else { 0 })?;
        self.pack_int(&mut buffer, replay.mods.value())?;
        self.pack_life_bar(&mut buffer, &replay.life_bar_graph)?;
        self.pack_timestamp(&mut buffer, &replay.timestamp)?;
        self.pack_replay_data(&mut buffer, &replay.replay_data, replay.rng_seed)?;
        self.pack_long(&mut buffer, replay.replay_id)?;

        Ok(buffer)
    }
}
