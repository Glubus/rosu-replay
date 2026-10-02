use super::{HitStatistics, LazerMod};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Metadata appended to lazer `.osr` files. Unknown fields survive read/write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LazerScoreInfo {
    #[serde(default)]
    pub client_version: String,
    #[serde(default = "offline_id")]
    pub online_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<String>,
    #[serde(default)]
    pub mods: Vec<LazerMod>,
    #[serde(default)]
    pub statistics: HitStatistics,
    #[serde(default)]
    pub maximum_statistics: HitStatistics,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_score_without_mods: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pauses: Option<Vec<i32>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
fn offline_id() -> i64 {
    -1
}
