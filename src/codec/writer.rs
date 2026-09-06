use super::compression;
use crate::{types::*, LazerScoreInfo, Replay, ReplayError};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// Shared stable/lazer binary writer.
pub struct Packer {
    preset: u32,
}
impl Default for Packer {
    fn default() -> Self {
        Self { preset: 6 }
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

        let ticks = i128::from(TICKS_TO_UNIX_EPOCH)
            + i128::from(unix_timestamp) * i128::from(TICKS_PER_SECOND)
            + i128::from(nanoseconds / 100);
        if !(0..=3_155_378_975_999_999_999).contains(&ticks) {
            return Err(ReplayError::InvalidFormat(
                "timestamp is outside the .NET DateTime range".into(),
            ));
        }
        self.pack_long(writer, ticks as i64)?;
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

    fn frames(replay_data: &[ReplayEvent], rng_seed: Option<i32>) -> String {
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

        data
    }
    fn write_block(&self, writer: &mut impl Write, data: &[u8]) -> Result<(), ReplayError> {
        let len = i32::try_from(data.len())
            .map_err(|_| ReplayError::InvalidFormat("block exceeds signed 32-bit length".into()))?;
        writer.write_i32::<LittleEndian>(len)?;
        writer.write_all(data)?;
        Ok(())
    }
    pub fn pack_lazer_score_info(
        &self,
        writer: &mut impl Write,
        info: &LazerScoreInfo,
    ) -> Result<(), ReplayError> {
        let compressed = compression::encode(&serde_json::to_vec(info)?, self.preset)?;
        self.write_block(writer, &compressed)
    }
    pub fn pack(&self, replay: &Replay) -> Result<Vec<u8>, ReplayError> {
        self.pack_inner(replay, true)
    }
    /// Diagnostic dump only: uncompressed frames are not valid in `.osr` files.
    pub fn pack_uncompressed(&self, replay: &Replay) -> Result<Vec<u8>, ReplayError> {
        self.pack_inner(replay, false)
    }
    fn pack_inner(&self, replay: &Replay, compress_frames: bool) -> Result<Vec<u8>, ReplayError> {
        replay.validate()?;
        let common = replay.common();
        let mut buffer = Vec::new();
        self.pack_byte(&mut buffer, common.mode as u8)?;
        self.pack_int(&mut buffer, replay.game_version())?;
        self.pack_string(&mut buffer, Some(&common.beatmap_hash))?;
        self.pack_string(&mut buffer, Some(&common.username))?;
        self.pack_string(&mut buffer, Some(&common.replay_hash))?;
        self.pack_short(&mut buffer, common.count_300)?;
        self.pack_short(&mut buffer, common.count_100)?;
        self.pack_short(&mut buffer, common.count_50)?;
        self.pack_short(&mut buffer, common.count_geki)?;
        self.pack_short(&mut buffer, common.count_katu)?;
        self.pack_short(&mut buffer, common.count_miss)?;
        self.pack_int(&mut buffer, common.score)?;
        self.pack_short(&mut buffer, common.max_combo)?;
        self.pack_byte(&mut buffer, if common.perfect { 1 } else { 0 })?;
        self.pack_int(&mut buffer, replay.legacy_mods().value())?;
        self.pack_life_bar(&mut buffer, &common.life_bar_graph)?;
        self.pack_timestamp(&mut buffer, &common.timestamp)?;

        let frames = Self::frames(&common.replay_data, common.rng_seed);
        let frames = if compress_frames {
            compression::encode(frames.as_bytes(), self.preset)?
        } else {
            frames.into_bytes()
        };
        self.write_block(&mut buffer, &frames)?;
        match replay {
            Replay::Stable(stable) => {
                if let Some(id) = stable.online_id() {
                    if stable.version().has_long_online_id() {
                        buffer.write_i64::<LittleEndian>(id)?;
                    } else {
                        buffer
                            .write_i32::<LittleEndian>(i32::try_from(id).map_err(|_| {
                                ReplayError::InvalidFormat("ID exceeds i32".into())
                            })?)?;
                    }
                }
            }
            Replay::Lazer(lazer) => {
                buffer.write_i64::<LittleEndian>(lazer.legacy_online_id())?;
                if lazer.version().has_score_info() {
                    if let Some(info) = lazer.score_info() {
                        self.pack_lazer_score_info(&mut buffer, info)?;
                    } else {
                        self.write_block(&mut buffer, &[])?;
                    }
                }
            }
        }
        Ok(buffer)
    }
}
