//! Lazer `.osr` replay metadata and lossless JSON types.
mod mods;
mod score;
mod statistics;
mod version;
use crate::{Mod, ReplayCommon, ReplayError};
pub use mods::LazerMod;
pub use score::LazerScoreInfo;
use serde::{Deserialize, Serialize};
pub use statistics::{HitResult, HitStatistics};
pub use version::LazerVersion;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "LazerReplayData")]
pub struct LazerReplay {
    common: ReplayCommon,
    version: LazerVersion,
    legacy_mods: Mod,
    legacy_online_id: i64,
    score_info: Option<LazerScoreInfo>,
}
impl LazerReplay {
    pub fn new(
        common: ReplayCommon,
        version: LazerVersion,
        legacy_mods: Mod,
        legacy_online_id: i64,
        score_info: Option<LazerScoreInfo>,
    ) -> Result<Self, ReplayError> {
        let replay = Self {
            common,
            version,
            legacy_mods,
            legacy_online_id,
            score_info,
        };
        replay.validate()?;
        Ok(replay)
    }
    pub fn common(&self) -> &ReplayCommon {
        &self.common
    }
    pub fn common_mut(&mut self) -> &mut ReplayCommon {
        &mut self.common
    }
    pub fn version(&self) -> LazerVersion {
        self.version
    }
    pub fn legacy_mods(&self) -> Mod {
        self.legacy_mods
    }
    pub fn legacy_online_id(&self) -> i64 {
        self.legacy_online_id
    }
    pub fn set_legacy_online_id(&mut self, id: i64) {
        self.legacy_online_id = id;
    }
    pub fn set_legacy_mods(&mut self, mods: Mod) {
        self.legacy_mods = mods;
    }
    pub fn score_info(&self) -> Option<&LazerScoreInfo> {
        self.score_info.as_ref()
    }
    pub fn score_info_mut(&mut self) -> Option<&mut LazerScoreInfo> {
        self.score_info.as_mut()
    }
    pub fn set_score_info(&mut self, info: Option<LazerScoreInfo>) -> Result<(), ReplayError> {
        if !self.version.has_score_info() && info.is_some() {
            return Err(ReplayError::InvalidFormat(
                "this lazer version has no score-info block".into(),
            ));
        }
        self.score_info = info;
        Ok(())
    }
    /// Interpret mods using this replay's ruleset. Unknown JSON remains preserved
    /// even if the installed rosu-mods version cannot interpret these settings.
    pub fn mods(&self) -> Result<Option<rosu_mods::GameMods>, ReplayError> {
        self.score_info
            .as_ref()
            .map(|info| mods::typed_mods(&info.mods, self.common.mode).map_err(Into::into))
            .transpose()
    }
    pub(crate) fn validate(&self) -> Result<(), ReplayError> {
        if !self.version.has_score_info() && self.score_info.is_some() {
            return Err(ReplayError::InvalidFormat(
                "this lazer version has no score-info block".into(),
            ));
        }
        self.common.validate()
    }
}
#[derive(Deserialize)]
struct LazerReplayData {
    common: ReplayCommon,
    version: LazerVersion,
    legacy_mods: Mod,
    legacy_online_id: i64,
    score_info: Option<LazerScoreInfo>,
}
impl TryFrom<LazerReplayData> for LazerReplay {
    type Error = ReplayError;
    fn try_from(v: LazerReplayData) -> Result<Self, Self::Error> {
        Self::new(
            v.common,
            v.version,
            v.legacy_mods,
            v.legacy_online_id,
            v.score_info,
        )
    }
}
