//! Mode-specific replay frames.
use crate::{GameMode, Key, KeyMania, KeyTaiko, ReplayError};
use serde::{Deserialize, Serialize};

/// A single event (frame) in a replay, specific to the game mode.
///
/// Each variant contains mode-specific information about what happened
/// at a particular time during the replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplayEvent {
    Osu(ReplayEventOsu),
    Taiko(ReplayEventTaiko),
    Catch(ReplayEventCatch),
    Mania(ReplayEventMania),
}

impl ReplayEvent {
    pub fn time_delta(&self) -> i32 {
        match self {
            ReplayEvent::Osu(event) => event.time_delta,
            ReplayEvent::Taiko(event) => event.time_delta,
            ReplayEvent::Catch(event) => event.time_delta,
            ReplayEvent::Mania(event) => event.time_delta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEventOsu {
    pub time_delta: i32,
    pub x: f32,
    pub y: f32,
    pub keys: Key,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEventTaiko {
    pub time_delta: i32,
    pub x: i32,
    pub keys: KeyTaiko,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEventCatch {
    pub time_delta: i32,
    pub x: f32,
    pub dashing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEventMania {
    pub time_delta: i32,
    pub keys: KeyMania,
}

pub(crate) fn parse_frames(
    replay_data_str: &str,
    mode: GameMode,
) -> Result<(Vec<ReplayEvent>, Option<i32>), ReplayError> {
    // Remove trailing comma if it exists
    let replay_data_str = replay_data_str.trim_end_matches(',');

    if replay_data_str.is_empty() {
        return Ok((Vec::new(), None));
    }

    let mut events = replay_data_str.split(',').peekable();
    let mut play_data = Vec::new();
    let mut rng_seed = None;

    while let Some(event_str) = events.next() {
        let mut parts = event_str.split('|');
        let (Some(delta), Some(x_str), Some(y_str), Some(keys_str), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) else {
            continue;
        };
        let time_delta = delta
            .parse::<i32>()
            .map_err(|e| ReplayError::Parse(format!("Invalid time_delta: {e}")))?;
        // The sentinel carries a signed RNG seed, not a key bitmask.
        if time_delta == -12345 && events.peek().is_none() {
            let seed = keys_str
                .parse::<i32>()
                .or_else(|_| keys_str.parse::<u32>().map(|v| v as i32))
                .map_err(|e| ReplayError::Parse(format!("Invalid RNG seed: {e}")))?;
            rng_seed = Some(seed);
            continue;
        }
        let keys = keys_str
            .parse::<u32>()
            .map_err(|e| ReplayError::Parse(format!("Invalid keys: {e}")))?;

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
