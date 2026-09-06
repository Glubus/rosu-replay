use crate::ReplayError;
use serde::{Deserialize, Serialize};

/// Version of the legacy-compatible lazer `.osr` container (not a client build date).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct LazerVersion(u32);
impl LazerVersion {
    pub fn new(value: u32) -> Result<Self, ReplayError> {
        if value < 30_000_000 {
            return Err(ReplayError::InvalidFormat(
                "lazer version must be at least 30000000".into(),
            ));
        }
        Ok(Self(value))
    }
    pub fn get(self) -> u32 {
        self.0
    }
    pub fn has_score_info(self) -> bool {
        self.0 >= 30_000_001
    }
}
impl TryFrom<u32> for LazerVersion {
    type Error = ReplayError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<LazerVersion> for u32 {
    fn from(value: LazerVersion) -> Self {
        value.0
    }
}
