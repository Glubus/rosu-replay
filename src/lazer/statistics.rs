use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Lazer judgement name, retaining names introduced by future clients.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum HitResult {
    None,
    Miss,
    Meh,
    Ok,
    Good,
    Great,
    Perfect,
    SmallTickMiss,
    SmallTickHit,
    LargeTickMiss,
    LargeTickHit,
    SmallBonus,
    LargeBonus,
    IgnoreMiss,
    IgnoreHit,
    ComboBreak,
    SliderTailHit,
    LegacyComboIncrease,
    Unknown(String),
}

impl From<String> for HitResult {
    fn from(name: String) -> Self {
        match name.as_str() {
            "none" => Self::None,
            "miss" => Self::Miss,
            "meh" => Self::Meh,
            "ok" => Self::Ok,
            "good" => Self::Good,
            "great" => Self::Great,
            "perfect" => Self::Perfect,
            "small_tick_miss" => Self::SmallTickMiss,
            "small_tick_hit" => Self::SmallTickHit,
            "large_tick_miss" => Self::LargeTickMiss,
            "large_tick_hit" => Self::LargeTickHit,
            "small_bonus" => Self::SmallBonus,
            "large_bonus" => Self::LargeBonus,
            "ignore_miss" => Self::IgnoreMiss,
            "ignore_hit" => Self::IgnoreHit,
            "combo_break" => Self::ComboBreak,
            "slider_tail_hit" => Self::SliderTailHit,
            "legacy_combo_increase" => Self::LegacyComboIncrease,
            _ => Self::Unknown(name),
        }
    }
}
impl From<HitResult> for String {
    fn from(value: HitResult) -> Self {
        match value {
            HitResult::None => "none",
            HitResult::Miss => "miss",
            HitResult::Meh => "meh",
            HitResult::Ok => "ok",
            HitResult::Good => "good",
            HitResult::Great => "great",
            HitResult::Perfect => "perfect",
            HitResult::SmallTickMiss => "small_tick_miss",
            HitResult::SmallTickHit => "small_tick_hit",
            HitResult::LargeTickMiss => "large_tick_miss",
            HitResult::LargeTickHit => "large_tick_hit",
            HitResult::SmallBonus => "small_bonus",
            HitResult::LargeBonus => "large_bonus",
            HitResult::IgnoreMiss => "ignore_miss",
            HitResult::IgnoreHit => "ignore_hit",
            HitResult::ComboBreak => "combo_break",
            HitResult::SliderTailHit => "slider_tail_hit",
            HitResult::LegacyComboIncrease => "legacy_combo_increase",
            HitResult::Unknown(name) => return name,
        }
        .into()
    }
}

/// Sparse counts; missing judgements read as zero without inserting JSON entries.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HitStatistics(pub BTreeMap<HitResult, u32>);
impl HitStatistics {
    pub fn count(&self, result: &HitResult) -> u32 {
        self.0.get(result).copied().unwrap_or(0)
    }
}
