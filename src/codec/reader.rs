/// Resource limits for untrusted `.osr` input. Sizes are bytes.
#[derive(Debug, Clone, Copy)]
pub struct ReadLimits {
    pub max_string_bytes: usize,
    pub max_compressed_bytes: usize,
    pub max_decompressed_frames: usize,
    pub max_decompressed_score: usize,
}
impl Default for ReadLimits {
    fn default() -> Self {
        Self {
            max_string_bytes: 16 * 1024 * 1024,
            max_compressed_bytes: 64 * 1024 * 1024,
            max_decompressed_frames: 256 * 1024 * 1024,
            max_decompressed_score: 16 * 1024 * 1024,
        }
    }
}

use super::compression;
use crate::{error::ReplayError, replay::Replay, types::*};
use crate::{LazerReplay, LazerScoreInfo, LazerVersion, ReplayCommon, StableReplay, StableVersion};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, TimeZone, Utc};
use std::io::Read;

/// Helper struct for unpacking .osr format data
pub struct Unpacker<R: Read> {
    reader: R,
    limits: ReadLimits,
}

impl<R: Read> Unpacker<R> {
    pub fn new(reader: R) -> Self {
        Self::with_limits(reader, ReadLimits::default())
    }

    pub fn with_limits(reader: R, limits: ReadLimits) -> Self {
        Self { reader, limits }
    }

    fn read_sized(&mut self, len: usize, limit: usize) -> Result<Vec<u8>, ReplayError> {
        if len > limit {
            return Err(ReplayError::InvalidFormat(
                "block exceeds configured limit".into(),
            ));
        }
        // Grow only as bytes arrive: a forged length must not eagerly allocate gigabytes.
        let mut data = Vec::new();
        self.reader
            .by_ref()
            .take(len as u64)
            .read_to_end(&mut data)?;
        if data.len() != len {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
        }
        Ok(data)
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
            let payload = usize::from(byte & 0x7f);
            if shift >= usize::BITS || payload > (usize::MAX >> shift) {
                return Err(ReplayError::InvalidFormat(
                    "ULEB128 length overflows usize".into(),
                ));
            }
            result |= payload << shift;

            if (byte & 0b10000000) == 0x00 {
                break;
            }

            shift += 7;
            if shift >= usize::BITS {
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
                let buffer = self.read_sized(length, self.limits.max_string_bytes)?;
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

        if !(0..=3_155_378_975_999_999_999).contains(&ticks) {
            return Err(ReplayError::InvalidFormat(
                "timestamp is outside the .NET DateTime range".into(),
            ));
        }
        let unix_ticks = ticks - TICKS_TO_UNIX_EPOCH;
        let unix_seconds = unix_ticks.div_euclid(TICKS_PER_SECOND);
        let nanoseconds = unix_ticks.rem_euclid(TICKS_PER_SECOND) * 100;
        Utc.timestamp_opt(unix_seconds, nanoseconds as u32)
            .single()
            .ok_or_else(|| ReplayError::InvalidFormat("invalid timestamp".into()))
    }

    pub fn unpack_play_data(
        &mut self,
        mode: GameMode,
    ) -> Result<(Vec<ReplayEvent>, Option<i32>), ReplayError> {
        let length = self.reader.read_i32::<LittleEndian>()?;
        if length <= 0 {
            return Ok((Vec::new(), None));
        }
        let compressed = self.read_sized(length as usize, self.limits.max_compressed_bytes)?;
        let buffer = compression::decode(&compressed, self.limits.max_decompressed_frames)?;

        let data_str = String::from_utf8(buffer)?;
        Self::parse_replay_data(&data_str, mode)
    }

    /// Parse frames without allocating a temporary vector for each event.
    pub fn parse_replay_data(
        data: &str,
        mode: GameMode,
    ) -> Result<(Vec<ReplayEvent>, Option<i32>), ReplayError> {
        crate::frame::parse_frames(data, mode)
    }

    pub fn unpack_replay_id(&mut self, game_version: u32) -> Result<i64, ReplayError> {
        if game_version >= 20140721 {
            self.unpack_long()
        } else if game_version >= 20121008 {
            Ok(i64::from(self.reader.read_i32::<LittleEndian>()?))
        } else {
            Ok(-1)
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

    pub fn unpack_lazer_score_info(&mut self) -> Result<Option<LazerScoreInfo>, ReplayError> {
        let length = self.reader.read_i32::<LittleEndian>()?;
        if length <= 0 {
            return Ok(None);
        }
        let compressed = self.read_sized(length as usize, self.limits.max_compressed_bytes)?;
        let buffer = compression::decode(&compressed, self.limits.max_decompressed_score)?;
        Ok(Some(serde_json::from_slice(&buffer)?))
    }

    pub fn unpack(mut self) -> Result<Replay, ReplayError> {
        let mode = match self.unpack_byte()? {
            0 => GameMode::Std,
            1 => GameMode::Taiko,
            2 => GameMode::Catch,
            3 => GameMode::Mania,
            value => {
                return Err(ReplayError::InvalidFormat(format!(
                    "unsupported game mode {value}"
                )))
            }
        };
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

        // named as `LegacyOnlineId` in lazer codebase
        let replay_id = self.unpack_replay_id(game_version)?;

        // https://github.com/ppy/osu/blob/48c4800e3ae4ee752452cdff83bd3787ccf3105f/osu.Game/Scoring/Legacy/LegacyScoreDecoder.cs#L117
        let lazer_score_info = if game_version >= 30000001 {
            self.unpack_lazer_score_info()?
        } else {
            None
        };

        let common = ReplayCommon {
            mode,
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
            life_bar_graph,
            timestamp,
            replay_data,
            rng_seed,
        };
        if game_version >= 30000000 {
            Ok(LazerReplay::new(
                common,
                LazerVersion::new(game_version)?,
                mods,
                replay_id,
                lazer_score_info,
            )?
            .into())
        } else {
            let version = StableVersion::new(game_version)?;
            Ok(StableReplay::new(
                common,
                version,
                mods,
                version.has_online_id().then_some(replay_id),
            )?
            .into())
        }
    }
}
