use crate::GameMode;
use rosu_mods::{serde::GameModsSeed, GameMods};
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use serde_json::{Map, Value};

/// Lossless JSON mod. The raw representation remains authoritative for writing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LazerMod {
    pub acronym: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub(crate) fn typed_mods(mods: &[LazerMod], mode: GameMode) -> Result<GameMods, serde_json::Error> {
    let mode = match mode {
        GameMode::Std => rosu_mods::GameMode::Osu,
        GameMode::Taiko => rosu_mods::GameMode::Taiko,
        GameMode::Catch => rosu_mods::GameMode::Catch,
        GameMode::Mania => rosu_mods::GameMode::Mania,
    };
    GameModsSeed::Mode {
        mode,
        deny_unknown_fields: false,
    }
    .deserialize(serde_json::to_value(mods)?)
}
