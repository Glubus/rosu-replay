use rosu_replay::{GameMode, Key, KeyMania, KeyTaiko, LifeBarState, Mod, Replay, ReplayEvent};

/// Test parsing basic replay data structures
#[test]
fn test_game_mode_conversion() {
    assert_eq!(GameMode::from(0), GameMode::Std);
    assert_eq!(GameMode::from(1), GameMode::Taiko);
    assert_eq!(GameMode::from(2), GameMode::Catch);
    assert_eq!(GameMode::from(3), GameMode::Mania);
    assert_eq!(GameMode::from(255), GameMode::Std); // Default fallback
}

#[test]
fn test_mod_operations() {
    let no_mod = Mod::NO_MOD;
    let hidden = Mod::HIDDEN;
    let hard_rock = Mod::HARD_ROCK;

    assert_eq!(no_mod.value(), 0);
    assert_eq!(hidden.value(), 1 << 3);
    assert_eq!(hard_rock.value(), 1 << 4);

    // Test mod combination
    let combined = Mod(hidden.value() | hard_rock.value());
    assert!(combined.contains(hidden));
    assert!(combined.contains(hard_rock));
    assert!(!combined.contains(Mod::EASY));
}

#[test]
fn test_key_values() {
    assert_eq!(Key::M1.value(), 1);
    assert_eq!(Key::M2.value(), 2);
    assert_eq!(Key::K1.value(), 4);
    assert_eq!(Key::K2.value(), 8);
    assert_eq!(Key::SMOKE.value(), 16);

    // Test combined keys
    let combined = Key(Key::M1.value() | Key::K1.value());
    assert_eq!(combined.value(), 5);
}

#[test]
fn test_taiko_keys() {
    assert_eq!(KeyTaiko::LEFT_DON.value(), 1);
    assert_eq!(KeyTaiko::LEFT_KAT.value(), 2);
    assert_eq!(KeyTaiko::RIGHT_DON.value(), 4);
    assert_eq!(KeyTaiko::RIGHT_KAT.value(), 8);
}

#[test]
fn test_mania_keys() {
    assert_eq!(KeyMania::K1.value(), 1);
    assert_eq!(KeyMania::K2.value(), 2);
    assert_eq!(KeyMania::K3.value(), 4);
    assert_eq!(KeyMania::K18.value(), 1 << 17);
}

/// Test creating a minimal valid replay
#[test]
fn test_create_minimal_replay() {
    let replay = create_test_replay();

    assert_eq!(replay.username, "TestPlayer");
    assert_eq!(replay.score, 1000000);
    assert_eq!(replay.mode, GameMode::Std);
    assert_eq!(replay.count_300, 100);
    assert_eq!(replay.replay_data.len(), 3);
}

/// Test replay serialization and deserialization
#[test]
fn test_replay_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let original_replay = create_test_replay();

    // Pack the replay
    let packed_data = original_replay.pack()?;
    assert!(!packed_data.is_empty());

    // Unpack the replay
    let unpacked_replay = Replay::from_bytes(&packed_data)?;

    // Verify basic fields match
    assert_eq!(original_replay.username, unpacked_replay.username);
    assert_eq!(original_replay.score, unpacked_replay.score);
    assert_eq!(original_replay.mode, unpacked_replay.mode);
    assert_eq!(original_replay.count_300, unpacked_replay.count_300);
    assert_eq!(original_replay.count_100, unpacked_replay.count_100);
    assert_eq!(original_replay.count_50, unpacked_replay.count_50);
    assert_eq!(original_replay.count_miss, unpacked_replay.count_miss);
    assert_eq!(original_replay.max_combo, unpacked_replay.max_combo);
    assert_eq!(original_replay.perfect, unpacked_replay.perfect);
    assert_eq!(original_replay.mods.value(), unpacked_replay.mods.value());
    assert_eq!(original_replay.replay_id, unpacked_replay.replay_id);

    // Verify replay data
    assert_eq!(
        original_replay.replay_data.len(),
        unpacked_replay.replay_data.len()
    );

    Ok(())
}

/// Test different game mode events
#[test]
fn test_game_mode_events() {
    // Test osu!standard event
    if let ReplayEvent::Osu(osu_event) = create_osu_event() {
        assert_eq!(osu_event.time_delta, 16);
        assert_eq!(osu_event.x, 256.0);
        assert_eq!(osu_event.y, 192.0);
        assert_eq!(osu_event.keys.value(), 1);
    } else {
        panic!("Expected osu event");
    }

    // Test taiko event
    if let ReplayEvent::Taiko(taiko_event) = create_taiko_event() {
        assert_eq!(taiko_event.time_delta, 32);
        assert_eq!(taiko_event.x, 320);
        assert_eq!(taiko_event.keys.value(), 1);
    } else {
        panic!("Expected taiko event");
    }

    // Test catch event
    if let ReplayEvent::Catch(catch_event) = create_catch_event() {
        assert_eq!(catch_event.time_delta, 20);
        assert_eq!(catch_event.x, 128.5);
        assert!(catch_event.dashing);
    } else {
        panic!("Expected catch event");
    }

    // Test mania event
    if let ReplayEvent::Mania(mania_event) = create_mania_event() {
        assert_eq!(mania_event.time_delta, 25);
        assert_eq!(mania_event.keys.value(), 5); // K1 + K3
    } else {
        panic!("Expected mania event");
    }
}

/// Test life bar data
#[test]
fn test_life_bar_data() {
     let life_states = [
        LifeBarState { time: 0, life: 1.0 },
        LifeBarState {
            time: 1000,
            life: 0.8,
        },
        LifeBarState {
            time: 2000,
            life: 0.6,
        },
        LifeBarState {
            time: 3000,
            life: 0.4,
        },
        LifeBarState {
            time: 4000,
            life: 0.2,
        },
    ];

    assert_eq!(life_states.len(), 5);
    assert_eq!(life_states[0].life, 1.0);
    assert_eq!(life_states[4].life, 0.2);
    assert_eq!(life_states[2].time, 2000);
}

/// Test error handling
#[test]
fn test_invalid_replay_data() {
    // Test empty data
    let result = Replay::from_bytes(&[]);
    assert!(result.is_err());

    // Test invalid data
    let invalid_data = vec![0xFF; 10];
    let result = Replay::from_bytes(&invalid_data);
    assert!(result.is_err());

    // Test truncated data
    let truncated_data = vec![0, 1, 2, 3];
    let result = Replay::from_bytes(&truncated_data);
    assert!(result.is_err());
}

/// Test replay data time calculation
#[test]
fn test_replay_time_calculation() {
    let events = [
        create_osu_event(),
        ReplayEvent::Osu(rosu_replay::ReplayEventOsu {
            time_delta: 50,
            x: 100.0,
            y: 100.0,
            keys: Key::M1,
        }),
        ReplayEvent::Osu(rosu_replay::ReplayEventOsu {
            time_delta: 33,
            x: 200.0,
            y: 200.0,
            keys: Key::M2,
        }),
    ];

    let total_time: i32 = events.iter().map(|e| e.time_delta()).sum();
    assert_eq!(total_time, 16 + 50 + 33); // 99ms total
}

// Helper functions for creating test data

fn create_test_replay() -> Replay {
    Replay {
        mode: GameMode::Std,
        game_version: 20240101,
        beatmap_hash: "abcdef1234567890".to_string(),
        username: "TestPlayer".to_string(),
        replay_hash: "fedcba0987654321".to_string(),
        count_300: 100,
        count_100: 10,
        count_50: 5,
        count_geki: 20,
        count_katu: 8,
        count_miss: 2,
        score: 1000000,
        max_combo: 150,
        perfect: false,
        mods: Mod::HIDDEN,
        life_bar_graph: Some(vec![
            LifeBarState { time: 0, life: 1.0 },
            LifeBarState {
                time: 10000,
                life: 0.5,
            },
            LifeBarState {
                time: 20000,
                life: 0.8,
            },
        ]),
        timestamp: chrono::Utc::now(),
        replay_data: vec![create_osu_event(), create_osu_event(), create_osu_event()],
        replay_id: 12345,
        rng_seed: Some(67890),
    }
}

fn create_osu_event() -> ReplayEvent {
    ReplayEvent::Osu(rosu_replay::ReplayEventOsu {
        time_delta: 16,
        x: 256.0,
        y: 192.0,
        keys: Key::M1,
    })
}

fn create_taiko_event() -> ReplayEvent {
    ReplayEvent::Taiko(rosu_replay::ReplayEventTaiko {
        time_delta: 32,
        x: 320,
        keys: KeyTaiko::LEFT_DON,
    })
}

fn create_catch_event() -> ReplayEvent {
    ReplayEvent::Catch(rosu_replay::ReplayEventCatch {
        time_delta: 20,
        x: 128.5,
        dashing: true,
    })
}

fn create_mania_event() -> ReplayEvent {
    ReplayEvent::Mania(rosu_replay::ReplayEventMania {
        time_delta: 25,
        keys: KeyMania(KeyMania::K1.value() | KeyMania::K3.value()),
    })
}
