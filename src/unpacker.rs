use crate::{error::ReplayError, replay::Replay, types::*};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, TimeZone, Utc};
use lzma_rs::lzma_decompress;
use std::io::Read;

/// Helper struct for unpacking .osr format data
pub struct Unpacker<R: Read> {
    reader: R,
}

impl<R: Read> Unpacker<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn unpack_byte(&mut self) -> Result<u8, ReplayError> {
        Ok(self.reader.read_u8()?)
    }

    pub fn unpack_short(&mut self) -> Result<u16, ReplayError> {
        Ok(self.reader.read_u16::<LittleEndian>()?)
    }

    pub fn unpack_int(&mut self) -> Result<u32, ReplayError> {
        Ok(self.reader.read_u32::<LittleEndian>()?)
    }

    pub fn unpack_long(&mut self) -> Result<i64, ReplayError> {
        Ok(self.reader.read_i64::<LittleEndian>()?)
    }

    fn read_uleb128(&mut self) -> Result<usize, ReplayError> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.reader.read_u8()?;
            result |= ((byte & 0b01111111) as usize) << shift;

            if (byte & 0b10000000) == 0x00 {
                break;
            }

            shift += 7;
            if shift >= 64 {
                return Err(ReplayError::InvalidFormat("ULEB128 too long".to_string()));
            }
        }

        Ok(result)
    }

    pub fn unpack_string(&mut self) -> Result<Option<String>, ReplayError> {
        let indicator = self.reader.read_u8()?;

        match indicator {
            0x00 => Ok(None),
            0x0b => {
                let length = self.read_uleb128()?;
                let mut buffer = vec![0u8; length];
                self.reader.read_exact(&mut buffer)?;
                let string = String::from_utf8(buffer)?;
                Ok(Some(string))
            }
            _ => Err(ReplayError::InvalidStringByte(indicator)),
        }
    }

    pub fn unpack_timestamp(&mut self) -> Result<DateTime<Utc>, ReplayError> {
        let ticks = self.unpack_long()?;

        // Windows ticks start from year 1 AD, Unix epoch starts from 1970
        // There are 621355968000000000 ticks between year 1 and Unix epoch
        const TICKS_TO_UNIX_EPOCH: i64 = 621355968000000000;
        const TICKS_PER_SECOND: i64 = 10_000_000;

        let unix_seconds = (ticks - TICKS_TO_UNIX_EPOCH) / TICKS_PER_SECOND;
        let nanoseconds = ((ticks - TICKS_TO_UNIX_EPOCH) % TICKS_PER_SECOND) * 100;

        Ok(Utc
            .timestamp_opt(unix_seconds, nanoseconds as u32)
            .single()
            .unwrap_or_else(Utc::now))
    }

    pub fn unpack_play_data(
        &mut self,
        mode: GameMode,
    ) -> Result<(Vec<ReplayEvent>, Option<i32>), ReplayError> {
        let replay_length = self.unpack_int()? as usize;
        let mut compressed_data = vec![0u8; replay_length];
        self.reader.read_exact(&mut compressed_data)?;

        let mut decompressed_data = Vec::new();
        lzma_decompress(&mut &compressed_data[..], &mut decompressed_data)
            .map_err(|e| ReplayError::Lzma(format!("{}", e)))?;
        let data_str = String::from_utf8(decompressed_data)?;

        Self::parse_replay_data(&data_str, mode)
    }

    pub fn parse_replay_data(
        replay_data_str: &str,
        mode: GameMode,
    ) -> Result<(Vec<ReplayEvent>, Option<i32>), ReplayError> {
        // Remove trailing comma if it exists
        let replay_data_str = replay_data_str.trim_end_matches(',');

        if replay_data_str.is_empty() {
            return Ok((Vec::new(), None));
        }

        let events: Vec<&str> = replay_data_str.split(',').collect();
        let mut play_data = Vec::new();
        let mut rng_seed = None;

        for (i, event_str) in events.iter().enumerate() {
            let parts: Vec<&str> = event_str.split('|').collect();
            if parts.len() != 4 {
                continue;
            }

            let time_delta = parts[0]
                .parse::<i32>()
                .map_err(|e| ReplayError::Parse(format!("Invalid time_delta: {}", e)))?;
            let x_str = parts[1];
            let y_str = parts[2];
            let keys = parts[3]
                .parse::<u32>()
                .map_err(|e| ReplayError::Parse(format!("Invalid keys: {}", e)))?;

            // Check for RNG seed (last event with special time_delta)
            if time_delta == -12345 && i == events.len() - 1 {
                rng_seed = Some(keys as i32);
                continue;
            }

            // Skip lazer frames with x=256, y=-500 in first two events
            if i < 2 {
                if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                    if x == 256.0 && y == -500.0 {
                        continue;
                    }
                }
            }

            let event = match mode {
                GameMode::Std => {
                    let x = x_str
                        .parse::<f32>()
                        .map_err(|e| ReplayError::Parse(format!("Invalid x coordinate: {}", e)))?;
                    let y = y_str
                        .parse::<f32>()
                        .map_err(|e| ReplayError::Parse(format!("Invalid y coordinate: {}", e)))?;
                    ReplayEvent::Osu(ReplayEventOsu {
                        time_delta,
                        x,
                        y,
                        keys: Key::from(keys),
                    })
                }
                GameMode::Taiko => {
                    let x = x_str
                        .parse::<i32>()
                        .map_err(|e| ReplayError::Parse(format!("Invalid x coordinate: {}", e)))?;
                    ReplayEvent::Taiko(ReplayEventTaiko {
                        time_delta,
                        x,
                        keys: KeyTaiko::from(keys),
                    })
                }
                GameMode::Catch => {
                    let x = x_str
                        .parse::<f32>()
                        .map_err(|e| ReplayError::Parse(format!("Invalid x coordinate: {}", e)))?;
                    ReplayEvent::Catch(ReplayEventCatch {
                        time_delta,
                        x,
                        dashing: keys == 1,
                    })
                }
                GameMode::Mania => {
                    let keys_value = x_str
                        .parse::<u32>()
                        .map_err(|e| ReplayError::Parse(format!("Invalid keys: {}", e)))?;
                    ReplayEvent::Mania(ReplayEventMania {
                        time_delta,
                        keys: KeyMania::from(keys_value),
                    })
                }
            };

            play_data.push(event);
        }

        Ok((play_data, rng_seed))
    }

    pub fn unpack_replay_id(&mut self) -> Result<i64, ReplayError> {
        // Try to read as long first, fallback to int for old replays
        match self.unpack_long() {
            Ok(id) => Ok(id),
            Err(_) => {
                // Reset and try as int
                Ok(self.unpack_int()? as i64)
            }
        }
    }

    pub fn unpack_life_bar(&mut self) -> Result<Option<Vec<LifeBarState>>, ReplayError> {
        let life_bar_string = self.unpack_string()?;

        match life_bar_string {
            None => Ok(None),
            Some(ref s) if s.is_empty() => Ok(None),
            Some(life_bar) => {
                let life_bar = life_bar.trim_end_matches(',');
                let states: Result<Vec<LifeBarState>, ReplayError> = life_bar
                    .split(',')
                    .map(|state_str| {
                        let parts: Vec<&str> = state_str.split('|').collect();
                        if parts.len() != 2 {
                            return Err(ReplayError::Parse(
                                "Invalid life bar state format".to_string(),
                            ));
                        }

                        let time = parts[0]
                            .parse::<i32>()
                            .map_err(|e| ReplayError::Parse(format!("Invalid time: {}", e)))?;
                        let life = parts[1]
                            .parse::<f32>()
                            .map_err(|e| ReplayError::Parse(format!("Invalid life: {}", e)))?;

                        Ok(LifeBarState { time, life })
                    })
                    .collect();

                Ok(Some(states?))
            }
        }
    }

    pub fn unpack(mut self) -> Result<Replay, ReplayError> {
        let mode = GameMode::from(self.unpack_byte()?);
        let game_version = self.unpack_int()?;
        let beatmap_hash = self.unpack_string()?.unwrap_or_default();
        let username = self.unpack_string()?.unwrap_or_default();
        let replay_hash = self.unpack_string()?.unwrap_or_default();
        let count_300 = self.unpack_short()?;
        let count_100 = self.unpack_short()?;
        let count_50 = self.unpack_short()?;
        let count_geki = self.unpack_short()?;
        let count_katu = self.unpack_short()?;
        let count_miss = self.unpack_short()?;
        let score = self.unpack_int()?;
        let max_combo = self.unpack_short()?;
        let perfect = self.unpack_byte()? != 0;
        let mods = Mod::from(self.unpack_int()?);
        let life_bar_graph = self.unpack_life_bar()?;
        let timestamp = self.unpack_timestamp()?;
        let (replay_data, rng_seed) = self.unpack_play_data(mode)?;
        let replay_id = self.unpack_replay_id()?;

        Ok(Replay {
            mode,
            game_version,
            beatmap_hash,
            username,
            replay_hash,
            count_300,
            count_100,
            count_50,
            count_geki,
            count_katu,
            count_miss,
            score,
            max_combo,
            perfect,
            mods,
            life_bar_graph,
            timestamp,
            replay_data,
            replay_id,
            rng_seed,
        })
    }
}
