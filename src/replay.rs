use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::Path;

use crate::{error::ReplayError, packer::Packer, types::*, unpacker::Unpacker};

/// Fields shared by stable and lazer binary headers and replay frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayCommon {
    /// The game mode this replay was played on
    pub mode: GameMode,
    /// The hash of the beatmap this replay was played on
    pub beatmap_hash: String,
    /// The user that played this replay
    pub username: String,
    /// The hash of this replay
    pub replay_hash: String,
    /// The number of 300 judgments in this replay
    pub count_300: u16,
    /// The number of 100 judgments in this replay
    pub count_100: u16,
    /// The number of 50 judgments in this replay
    pub count_50: u16,
    /// The number of geki judgments in this replay
    pub count_geki: u16,
    /// The number of katu judgments in this replay
    pub count_katu: u16,
    /// The number of misses in this replay
    pub count_miss: u16,
    /// The score of this replay
    pub score: u32,
    /// The maximum combo attained in this replay
    pub max_combo: u16,
    /// Whether this replay was perfect or not
    pub perfect: bool,
    /// The life bar of this replay over time
    pub life_bar_graph: Option<Vec<LifeBarState>>,
    /// The timestamp when this replay was played
    pub timestamp: DateTime<Utc>,
    /// The replay data of the replay, including cursor position and keys pressed
    pub replay_data: Vec<ReplayEvent>,
    /// The rng seed of this replay, or None if not present
    pub rng_seed: Option<i32>,
}

impl ReplayCommon {
    pub(crate) fn validate(&self) -> Result<(), ReplayError> {
        for frame in &self.replay_data {
            let mode = match frame {
                ReplayEvent::Osu(_) => GameMode::Std,
                ReplayEvent::Taiko(_) => GameMode::Taiko,
                ReplayEvent::Catch(_) => GameMode::Catch,
                ReplayEvent::Mania(_) => GameMode::Mania,
            };
            if mode != self.mode {
                return Err(ReplayError::InvalidFormat(
                    "frame ruleset differs from replay mode".into(),
                ));
            }
        }
        Ok(())
    }
}

/// A stable or lazer replay. The version selects its binary layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Replay {
    Stable(crate::StableReplay),
    Lazer(crate::LazerReplay),
}

impl Replay {
    pub fn common(&self) -> &ReplayCommon {
        match self {
            Self::Stable(v) => v.common(),
            Self::Lazer(v) => v.common(),
        }
    }
    /// Edited frame modes are validated before writing.
    pub fn common_mut(&mut self) -> &mut ReplayCommon {
        match self {
            Self::Stable(v) => v.common_mut(),
            Self::Lazer(v) => v.common_mut(),
        }
    }
    pub fn game_version(&self) -> u32 {
        match self {
            Self::Stable(v) => v.version().get(),
            Self::Lazer(v) => v.version().get(),
        }
    }
    /// Legacy bitflags from the binary header; lazer's full mods live in score_info.
    pub fn legacy_mods(&self) -> Mod {
        match self {
            Self::Stable(v) => v.mods(),
            Self::Lazer(v) => v.legacy_mods(),
        }
    }
    /// Preserves the raw signed ID, including zero/offline sentinels.
    pub fn legacy_online_id(&self) -> Option<i64> {
        match self {
            Self::Stable(v) => v.online_id(),
            Self::Lazer(v) => Some(v.legacy_online_id()),
        }
    }
    pub(crate) fn validate(&self) -> Result<(), ReplayError> {
        match self {
            Self::Stable(v) => v.validate(),
            Self::Lazer(v) => v.validate(),
        }
    }
}

impl From<crate::StableReplay> for Replay {
    fn from(value: crate::StableReplay) -> Self {
        Self::Stable(value)
    }
}
impl From<crate::LazerReplay> for Replay {
    fn from(value: crate::LazerReplay) -> Self {
        Self::Lazer(value)
    }
}

impl Replay {
    /// Creates a new `Replay` object from the `.osr` file at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the osr file to read from
    ///
    /// # Returns
    ///
    /// The parsed replay object
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ReplayError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    /// Creates a new `Replay` object from a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from
    ///
    /// # Returns
    ///
    /// The parsed replay object
    pub fn from_reader<R: std::io::Read>(reader: R) -> Result<Self, ReplayError> {
        let unpacker = Unpacker::new(reader);
        unpacker.unpack()
    }

    /// Read with explicit allocation and decompression limits.
    pub fn from_reader_with_limits<R: std::io::Read>(
        reader: R,
        limits: crate::ReadLimits,
    ) -> Result<Self, ReplayError> {
        Unpacker::with_limits(reader, limits).unpack()
    }

    /// Creates a new `Replay` object from a byte slice containing `.osr` data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to parse
    ///
    /// # Returns
    ///
    /// The parsed replay object
    pub fn from_bytes(data: &[u8]) -> Result<Self, ReplayError> {
        let cursor = Cursor::new(data);
        Self::from_reader(cursor)
    }

    /// Writes the replay to the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to where to write the replay
    ///
    /// # Notes
    ///
    /// This uses the current values of any attributes, and so can be used to
    /// create an edited version of a replay, by first reading a replay, editing
    /// an attribute, then writing the replay back to its file.
    pub fn write_path<P: AsRef<Path>>(&self, path: P) -> Result<(), ReplayError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        self.write_to(writer)
    }

    /// Writes the replay to a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write to
    pub fn write_to<W: std::io::Write>(&self, mut writer: W) -> Result<(), ReplayError> {
        let packed = self.pack()?;
        writer.write_all(&packed)?;
        Ok(())
    }

    /// Returns the bytes representing this `Replay`, in `.osr` format.
    ///
    /// The bytes returned by this method are suitable for writing to a file as a
    /// valid `.osr` file.
    ///
    /// # Returns
    ///
    /// The bytes representing this `Replay`, in `.osr` format
    pub fn pack(&self) -> Result<Vec<u8>, ReplayError> {
        let packer = Packer::new();
        packer.pack(self)
    }

    /// Returns the bytes representing this `Replay`, in `.osr` format, with custom packer settings.
    ///
    /// # Arguments
    ///
    /// * `packer` - The packer to use for compression settings
    ///
    /// # Returns
    ///
    /// The bytes representing this `Replay`, in `.osr` format
    pub fn pack_with(&self, packer: &Packer) -> Result<Vec<u8>, ReplayError> {
        packer.pack(self)
    }

    /// Diagnostic binary dump with uncompressed frames. This is NOT a valid `.osr`.
    ///
    /// This method is similar to `pack` but saves the replay data in uncompressed format,
    /// which can be useful for debugging or when you need faster processing at the cost
    /// of larger file size.
    ///
    /// # Returns
    ///
    /// Diagnostic bytes with raw frames and the normal version-dependent suffix
    pub fn pack_uncompressed(&self) -> Result<Vec<u8>, ReplayError> {
        let packer = Packer::new();
        packer.pack_uncompressed(self)
    }

    /// Diagnostic dump with custom packer settings. This is NOT a valid `.osr`.
    ///
    /// # Arguments
    ///
    /// * `packer` - The packer to use for compression settings
    ///
    /// # Returns
    ///
    /// Diagnostic bytes with raw frames and the normal version-dependent suffix
    pub fn pack_uncompressed_with(&self, packer: &Packer) -> Result<Vec<u8>, ReplayError> {
        packer.pack_uncompressed(self)
    }
}

/// Parses the replay data portion of a replay from a string.
///
/// This method is suitable for use with the replay data returned by API v1's
/// `/get_replay` endpoint, for instance.
///
/// # Arguments
///
/// * `data_string` - The replay data to parse
/// * `decoded` - Whether `data_string` has already been decoded from a base64 representation
/// * `decompressed` - Whether `data_string` has already been decompressed from lzma and decoded to ascii
/// * `mode` - What mode to parse the replay data as
///
/// # Returns
///
/// The parsed replay events
pub fn parse_replay_data(
    data_string: &[u8],
    decoded: bool,
    decompressed: bool,
    mode: GameMode,
) -> Result<Vec<ReplayEvent>, ReplayError> {
    let data = if !decoded && !decompressed {
        general_purpose::STANDARD
            .decode(data_string)
            .map_err(|e| ReplayError::Parse(format!("Base64 decode error: {}", e)))?
    } else {
        data_string.to_vec()
    };

    let decompressed_data = if !decompressed {
        crate::codec::compression::decode(
            &data,
            crate::ReadLimits::default().max_decompressed_frames,
        )?
    } else {
        data
    };

    let data_string = String::from_utf8(decompressed_data)?;
    let (replay_data, _) = Unpacker::<Cursor<&[u8]>>::parse_replay_data(&data_string, mode)?;

    Ok(replay_data)
}
