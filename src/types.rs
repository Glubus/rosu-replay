//! Core types and enums for osu! replay data.
//!
//! This module defines all the data structures used to represent osu! replay information,
//! including game modes, mods, key states, and replay events for different game modes.

use serde::{Deserialize, Serialize};

/// Represents the different game modes in osu!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// osu!standard - Traditional circle-clicking mode
    Std = 0,
    /// osu!taiko - Drum-based rhythm mode
    Taiko = 1,
    /// osu!catch - Fruit-catching mode
    Catch = 2,
    /// osu!mania - Piano-style rhythm mode
    Mania = 3,
}

impl From<u8> for GameMode {
    fn from(value: u8) -> Self {
        match value {
            0 => GameMode::Std,
            1 => GameMode::Taiko,
            2 => GameMode::Catch,
            3 => GameMode::Mania,
            _ => GameMode::Std, // Default fallback
        }
    }
}

/// Represents osu! mods as a bitflag integer.
///
/// Mods can be combined using bitwise OR operations.
///
/// # Example
///
/// ```rust
/// use rosu_replay::Mod;
///
/// let hidden_hard_rock = Mod::HIDDEN.0 | Mod::HARD_ROCK.0;
/// let combined_mod = Mod(hidden_hard_rock);
/// assert!(combined_mod.contains(Mod::HIDDEN));
/// assert!(combined_mod.contains(Mod::HARD_ROCK));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mod(pub u32);

impl Mod {
    pub const NO_MOD: Self = Self(0);
    pub const NO_FAIL: Self = Self(1 << 0);
    pub const EASY: Self = Self(1 << 1);
    pub const TOUCH_DEVICE: Self = Self(1 << 2);
    pub const HIDDEN: Self = Self(1 << 3);
    pub const HARD_ROCK: Self = Self(1 << 4);
    pub const SUDDEN_DEATH: Self = Self(1 << 5);
    pub const DOUBLE_TIME: Self = Self(1 << 6);
    pub const RELAX: Self = Self(1 << 7);
    pub const HALF_TIME: Self = Self(1 << 8);
    pub const NIGHTCORE: Self = Self(1 << 9);
    pub const FLASHLIGHT: Self = Self(1 << 10);
    pub const AUTOPLAY: Self = Self(1 << 11);
    pub const SPUN_OUT: Self = Self(1 << 12);
    pub const AUTOPILOT: Self = Self(1 << 13);
    pub const PERFECT: Self = Self(1 << 14);
    pub const KEY4: Self = Self(1 << 15);
    pub const KEY5: Self = Self(1 << 16);
    pub const KEY6: Self = Self(1 << 17);
    pub const KEY7: Self = Self(1 << 18);
    pub const KEY8: Self = Self(1 << 19);
    pub const FADE_IN: Self = Self(1 << 20);
    pub const RANDOM: Self = Self(1 << 21);
    pub const CINEMA: Self = Self(1 << 22);
    pub const TARGET: Self = Self(1 << 23);
    pub const KEY9: Self = Self(1 << 24);
    pub const KEY_COOP: Self = Self(1 << 25);
    pub const KEY1: Self = Self(1 << 26);
    pub const KEY3: Self = Self(1 << 27);
    pub const KEY2: Self = Self(1 << 28);
    pub const SCORE_V2: Self = Self(1 << 29);
    pub const MIRROR: Self = Self(1 << 30);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Mod {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Represents keys that can be pressed during osu!standard gameplay.
/// Includes mouse buttons (M1, M2), keyboard keys (K1, K2), and smoke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key(pub u32);

impl Key {
    pub const M1: Self = Self(1 << 0);
    pub const M2: Self = Self(1 << 1);
    pub const K1: Self = Self(1 << 2);
    pub const K2: Self = Self(1 << 3);
    pub const SMOKE: Self = Self(1 << 4);

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Key {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Represents keys that can be pressed during osu!taiko gameplay.
/// Includes different drum hit types for left and right sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyTaiko(pub u32);

impl KeyTaiko {
    pub const LEFT_DON: Self = Self(1 << 0);
    pub const LEFT_KAT: Self = Self(1 << 1);
    pub const RIGHT_DON: Self = Self(1 << 2);
    pub const RIGHT_KAT: Self = Self(1 << 3);

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for KeyTaiko {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Represents keys that can be pressed during osu!mania gameplay.
/// Supports up to 18 lanes (K1-K18) for different key configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyMania(pub u32);

impl KeyMania {
    pub const K1: Self = Self(1 << 0);
    pub const K2: Self = Self(1 << 1);
    pub const K3: Self = Self(1 << 2);
    pub const K4: Self = Self(1 << 3);
    pub const K5: Self = Self(1 << 4);
    pub const K6: Self = Self(1 << 5);
    pub const K7: Self = Self(1 << 6);
    pub const K8: Self = Self(1 << 7);
    pub const K9: Self = Self(1 << 8);
    pub const K10: Self = Self(1 << 9);
    pub const K11: Self = Self(1 << 10);
    pub const K12: Self = Self(1 << 11);
    pub const K13: Self = Self(1 << 12);
    pub const K14: Self = Self(1 << 13);
    pub const K15: Self = Self(1 << 14);
    pub const K16: Self = Self(1 << 15);
    pub const K17: Self = Self(1 << 16);
    pub const K18: Self = Self(1 << 17);

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for KeyMania {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

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

/// Represents the life bar state at a specific point in time during a replay.
///
/// The life bar shows the player's health throughout the song,
/// typically ranging from 0.0 (empty) to 1.0 (full).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifeBarState {
    pub time: i32,
    pub life: f32,
}
