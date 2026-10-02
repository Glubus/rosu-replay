//! Stable replay metadata. Binary primitives are shared with lazer in `codec`.
mod version;
use crate::{Mod, ReplayCommon, ReplayError};
use serde::{Deserialize, Serialize};
pub use version::StableVersion;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "StableReplayData")]
pub struct StableReplay {
    common: ReplayCommon,
    version: StableVersion,
    mods: Mod,
    online_id: Option<i64>,
}

impl StableReplay {
    pub fn new(
        common: ReplayCommon,
        version: StableVersion,
        mods: Mod,
        online_id: Option<i64>,
    ) -> Result<Self, ReplayError> {
        let replay = Self {
            common,
            version,
            mods,
            online_id,
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
    pub fn version(&self) -> StableVersion {
        self.version
    }
    pub fn mods(&self) -> Mod {
        self.mods
    }
    pub fn set_mods(&mut self, mods: Mod) {
        self.mods = mods;
    }
    /// Raw ID: `None` means no field in this version; zero and negative values are preserved.
    pub fn online_id(&self) -> Option<i64> {
        self.online_id
    }
    pub fn set_online_id(&mut self, id: Option<i64>) -> Result<(), ReplayError> {
        Self::validate_id(self.version, id)?;
        self.online_id = id;
        Ok(())
    }
    fn validate_id(version: StableVersion, id: Option<i64>) -> Result<(), ReplayError> {
        if version.has_online_id() != id.is_some() {
            return Err(ReplayError::InvalidFormat(
                "online ID presence does not match stable version".into(),
            ));
        }
        if !version.has_long_online_id() && id.is_some_and(|id| i32::try_from(id).is_err()) {
            return Err(ReplayError::InvalidFormat(
                "online ID exceeds this version's signed 32-bit field".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn validate(&self) -> Result<(), ReplayError> {
        Self::validate_id(self.version, self.online_id)?;
        self.common.validate()
    }
}

#[derive(Deserialize)]
struct StableReplayData {
    common: ReplayCommon,
    version: StableVersion,
    mods: Mod,
    online_id: Option<i64>,
}
impl TryFrom<StableReplayData> for StableReplay {
    type Error = ReplayError;
    fn try_from(v: StableReplayData) -> Result<Self, Self::Error> {
        Self::new(v.common, v.version, v.mods, v.online_id)
    }
}
