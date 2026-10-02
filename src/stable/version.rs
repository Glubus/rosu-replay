use crate::ReplayError;
use serde::{Deserialize, Serialize};

/// A stable `.osr` version, including old versions without an online ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct StableVersion(u32);

impl StableVersion {
    pub fn new(value: u32) -> Result<Self, ReplayError> {
        if value >= 30_000_000 {
            return Err(ReplayError::InvalidFormat(
                "stable version must be below 30000000".into(),
            ));
        }
        Ok(Self(value))
    }
    pub fn get(self) -> u32 {
        self.0
    }
    pub fn has_online_id(self) -> bool {
        self.0 >= 20_121_008
    }
    pub fn has_long_online_id(self) -> bool {
        self.0 >= 20_140_721
    }
}
impl TryFrom<u32> for StableVersion {
    type Error = ReplayError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<StableVersion> for u32 {
    fn from(value: StableVersion) -> Self {
        value.0
    }
}
