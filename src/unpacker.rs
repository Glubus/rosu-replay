use crate::{error::ReplayError, replay::Replay, types::*};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, TimeZone, Utc};
use liblzma::read;
use std::io::Read;

/// Upper bound on any decompressed block (play data, lazer score info).
/// Real replays are a few MB at most; this only stops decompression bombs.
pub(crate) const MAX_DECOMPRESSED_LEN: u64 = 256 * 1024 * 1024;

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

    /// Reads exactly `len` bytes without trusting `len` for the allocation:
    /// the buffer only grows as data actually arrives, so a forged length in a
    /// tiny file cannot trigger a huge allocation.
    fn read_block(&mut self, len: u64) -> Result<Vec<u8>, ReplayError> {
        let mut buffer = Vec::new();
        let read = (&mut self.reader).take(len).read_to_end(&mut buffer)?;
        if read as u64 != len {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
        }
        Ok(buffer)
    }

    /// Decompresses a LZMA/XZ block, refusing to expand past `limit` bytes.
    pub(crate) fn decompress(data: &[u8], limit: u64) -> Result<Vec<u8>, ReplayError> {
        let mut buffer = Vec::new();
        let read = read::XzDecoder::new_multi_decoder(data)
            .take(limit + 1)
            .read_to_end(&mut buffer)?;
        if read as u64 > limit {
            return Err(ReplayError::InvalidFormat(format!(
                "decompressed data exceeds {limit} bytes"
            )));
        }
        Ok(buffer)
    }

    fn read_uleb128(&mut self) -> Result<usize, ReplayError> {
        let mut result: u64 = 0;
        let mut shift = 0;

        loop {
            let byte = self.reader.read_u8()?;
            let bits = u64::from(byte & 0b01111111);

            // At shift 63 only one bit still fits in a u64.
            if shift == 63 && bits > 1 {
                return Err(ReplayError::InvalidFormat(
                    "ULEB128 overflows u64".to_string(),
                ));
            }
            result |= bits << shift;

            if (byte & 0b10000000) == 0x00 {
                break;
            }

            shift += 7;
            if shift >= 64 {
                return Err(ReplayError::InvalidFormat("ULEB128 too long".to_string()));
            }
        }

        usize::try_from(result)
            .map_err(|_| ReplayError::InvalidFormat("ULEB128 value too large".to_string()))
    }

    pub fn unpack_string(&mut self) -> Result<Option<String>, ReplayError> {
        let indicator = self.reader.read_u8()?;

        match indicator {
            0x00 => Ok(None),
            0x0b => {
                let length = self.read_uleb128()?;
                let buffer = self.read_block(length as u64)?;
                Ok(Some(String::from_utf8(buffer)?))
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

        // Saturate instead of overflowing on hostile tick values, and use
        // euclidean division so pre-epoch timestamps get non-negative nanoseconds.
        let since_epoch = ticks.saturating_sub(TICKS_TO_UNIX_EPOCH);
        let unix_seconds = since_epoch.div_euclid(TICKS_PER_SECOND);
        let nanoseconds = (since_epoch.rem_euclid(TICKS_PER_SECOND) * 100) as u32;

        Ok(Utc
            .timestamp_opt(unix_seconds, nanoseconds)
            .single()
            .unwrap_or_else(Utc::now))
    }

    pub fn unpack_play_data(
        &mut self,
        mode: GameMode,
    ) -> Result<(Vec<ReplayEvent>, Option<i32>), ReplayError> {
        let replay_length = self.unpack_int()?;
        let compressed_data = self.read_block(u64::from(replay_length))?;
        let data_str =
            String::from_utf8(Self::decompress(&compressed_data, MAX_DECOMPRESSED_LEN)?)?;
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

    pub fn unpack_replay_id(&mut self, game_version: u32) -> Result<i64, ReplayError> {
        // https://github.com/ppy/osu/blob/48c4800e3ae4ee752452cdff83bd3787ccf3105f/osu.Game/Scoring/Legacy/LegacyScoreDecoder.cs#L107-L113
        let replay_id = if game_version >= 20140721 {
            self.unpack_long()?
        } else if game_version >= 20121008 {
            self.unpack_int().map(|v| v as i64)?
        } else {
            0
        };

        if replay_id == 0 {
            Ok(-1)
        } else {
            Ok(replay_id)
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
        // The block is optional: no bytes at all means it is absent. A partial
        // length field is truncation and any other IO error is real, so both
        // must surface.
        let mut len_bytes = Vec::with_capacity(4);
        (&mut self.reader).take(4).read_to_end(&mut len_bytes)?;
        if len_bytes.is_empty() {
            return Ok(None);
        }
        let len_bytes: [u8; 4] = len_bytes
            .try_into()
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?;
        let len = u32::from_le_bytes(len_bytes);

        let compressed_data = self.read_block(u64::from(len))?;
        let data_str =
            String::from_utf8(Self::decompress(&compressed_data, MAX_DECOMPRESSED_LEN)?)?;
        Ok(Some(serde_json::from_str(&data_str)?))
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

        // named as `LegacyOnlineId` in lazer codebase
        let replay_id = self.unpack_replay_id(game_version)?;

        // https://github.com/ppy/osu/blob/48c4800e3ae4ee752452cdff83bd3787ccf3105f/osu.Game/Scoring/Legacy/LegacyScoreDecoder.cs#L117
        let lazer_score_info = if game_version >= 30000001 {
            self.unpack_lazer_score_info()?
        } else {
            None
        };

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
            lazer_score_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn compress(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut enc = liblzma::write::XzEncoder::new(&mut out, 1);
        enc.write_all(data).unwrap();
        enc.finish().unwrap();
        out
    }

    #[test]
    fn decompress_rejects_output_over_limit() {
        let bomb = compress(&vec![0u8; 4096]);
        assert!(bomb.len() < 200, "test payload should be a tiny bomb");

        assert_eq!(
            Unpacker::<&[u8]>::decompress(&bomb, 4096).unwrap().len(),
            4096
        );
        assert!(matches!(
            Unpacker::<&[u8]>::decompress(&bomb, 4095),
            Err(ReplayError::InvalidFormat(_))
        ));
    }

    #[test]
    fn forged_block_length_is_eof_not_allocation() {
        // Claims 4 GiB - 1 but provides 3 bytes.
        let mut data = Vec::new();
        data.extend_from_slice(&u32::MAX.to_le_bytes());
        data.extend_from_slice(&[1, 2, 3]);
        let mut unpacker = Unpacker::new(Cursor::new(data));
        let len = unpacker.unpack_int().unwrap();
        match unpacker.read_block(u64::from(len)) {
            Err(ReplayError::Io(e)) => assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof),
            other => panic!("expected UnexpectedEof, got {other:?}"),
        }
    }

    #[test]
    fn string_with_forged_length_errors() {
        let mut data = vec![0x0b];
        data.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]); // ~4 GiB
        data.extend_from_slice(b"abc");
        assert!(Unpacker::new(Cursor::new(data)).unpack_string().is_err());
    }

    #[test]
    fn uleb128_overflowing_u64_is_rejected() {
        // 10th byte carries 7 payload bits, only 1 fits.
        let mut data = vec![0xff; 9];
        data.push(0x7f);
        assert!(matches!(
            Unpacker::new(Cursor::new(data)).read_uleb128(),
            Err(ReplayError::InvalidFormat(_))
        ));
        // u64::MAX is the largest legal value.
        let mut max = vec![0xff; 9];
        max.push(0x01);
        let got = Unpacker::new(Cursor::new(max)).read_uleb128();
        assert_eq!(got.ok(), usize::try_from(u64::MAX).ok());
    }

    #[test]
    fn timestamp_extremes_do_not_panic() {
        for ticks in [i64::MIN, i64::MIN + 1, -1, 0, i64::MAX] {
            let mut u = Unpacker::new(Cursor::new(ticks.to_le_bytes().to_vec()));
            u.unpack_timestamp().unwrap();
        }
    }

    #[test]
    fn pre_epoch_timestamp_keeps_subsecond_part() {
        const TICKS_TO_UNIX_EPOCH: i64 = 621355968000000000;
        // 1969-12-31T23:59:59.5
        let ticks = TICKS_TO_UNIX_EPOCH - 5_000_000;
        let mut u = Unpacker::new(Cursor::new(ticks.to_le_bytes().to_vec()));
        let ts = u.unpack_timestamp().unwrap();
        assert_eq!(ts.timestamp(), -1);
        assert_eq!(ts.timestamp_subsec_nanos(), 500_000_000);
    }

    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        }
    }

    #[test]
    fn lazer_block_absent_at_eof_but_io_errors_propagate() {
        let mut empty = Unpacker::new(Cursor::new(Vec::new()));
        assert!(empty.unpack_lazer_score_info().unwrap().is_none());

        let mut failing = Unpacker::new(FailingReader);
        assert!(matches!(
            failing.unpack_lazer_score_info(),
            Err(ReplayError::Io(_))
        ));
    }

    #[test]
    fn lazer_block_with_partial_length_is_truncation() {
        for n in 1..4 {
            let mut u = Unpacker::new(Cursor::new(vec![0u8; n]));
            match u.unpack_lazer_score_info() {
                Err(ReplayError::Io(e)) => {
                    assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof)
                }
                other => panic!("{n} byte(s): expected UnexpectedEof, got {other:?}"),
            }
        }
    }
}
