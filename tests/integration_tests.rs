mod support;
use rosu_replay::{GameMode, HitResult, Packer, Replay, ReplayEvent};
use support::*;

#[test]
fn checked_in_replays_preserve_every_field_and_frame() {
    for bytes in [
        include_bytes!("../assets/test.osr").as_slice(),
        include_bytes!("../assets/test_lazer.osr").as_slice(),
    ] {
        let original = Replay::from_bytes(bytes).unwrap();
        assert!(!original.common().replay_data.is_empty());
        let rewritten = original.pack().unwrap();
        assert_eq!(Replay::from_bytes(&rewritten).unwrap(), original);
        if matches!(original, Replay::Lazer(_)) {
            // Independent LZMA/JSON decode also detects fields lost on FIRST read.
            assert_eq!(read_json(&rewritten), read_json(bytes));
        }
    }
}

#[test]
fn binary_header_values_are_read_at_the_correct_offsets() {
    let bytes = fixture(
        0,
        20140721,
        1234567890123,
        "16|256|192|5,-12345|0|0|42",
        &[],
    );
    let replay = Replay::from_bytes(&bytes).unwrap();
    assert!(matches!(replay, Replay::Stable(_)));
    let c = replay.common();
    assert_eq!(c.mode, GameMode::Std);
    assert_eq!(
        [
            c.count_300,
            c.count_100,
            c.count_50,
            c.count_geki,
            c.count_katu,
            c.count_miss
        ],
        [12, 2, 1, 3, 4, 5]
    );
    assert_eq!(c.score, 123456);
    assert_eq!(c.max_combo, 42);
    assert!(!c.perfect);
    assert_eq!(c.timestamp.timestamp_subsec_nanos(), 123456700);
    assert_eq!(replay.legacy_online_id(), Some(1234567890123));
    assert_eq!(c.rng_seed, Some(42));
    assert!(
        matches!(&c.replay_data[..], [ReplayEvent::Osu(f)] if f.time_delta == 16 && f.x == 256.0 && f.y == 192.0 && f.keys.value() == 5)
    );
}

#[test]
fn lazer_fixture_has_actual_mod_settings_and_statistics() {
    let bytes = include_bytes!("../assets/test_lazer.osr");
    let original_json = read_json(bytes);
    let Replay::Lazer(lazer) = Replay::from_bytes(bytes).unwrap() else {
        panic!("expected lazer")
    };
    let mods = lazer.mods().unwrap().unwrap();
    assert_eq!(mods.clock_rate(), Some(1.2));
    assert!(mods.iter().any(|m| matches!(m, rosu_mods::GameMod::AccuracyChallengeOsu(v) if v.minimum_accuracy == Some(0.95))));
    let info = lazer.score_info().unwrap();
    assert_eq!(
        serde_json::to_value(&info.statistics).unwrap(),
        original_json["statistics"]
    );
    assert_eq!(
        serde_json::to_value(&info.maximum_statistics).unwrap(),
        original_json["maximum_statistics"]
    );
    assert_eq!(
        info.statistics
            .count(&HitResult::Unknown("unrecorded".into())),
        0
    );
}

#[test]
fn compression_presets_preserve_data() {
    let replay = Replay::from_bytes(include_bytes!("../assets/test_lazer.osr")).unwrap();
    for preset in [0, 6, 9] {
        let packed = replay
            .pack_with(&Packer::new().with_preset(preset))
            .unwrap();
        assert_eq!(Replay::from_bytes(&packed).unwrap(), replay);
    }
}

#[test]
fn fragmented_reader_does_not_change_the_result() {
    struct Chunks<'a>(&'a [u8]);
    impl std::io::Read for Chunks<'_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            let n = out.len().min(3).min(self.0.len());
            out[..n].copy_from_slice(&self.0[..n]);
            self.0 = &self.0[n..];
            Ok(n)
        }
    }
    let bytes = fixture(
        3,
        30000001,
        -1,
        "16|5|0|0,-12345|0|0|42",
        &json_tail(&score()),
    );
    assert_eq!(
        Replay::from_reader(Chunks(&bytes)).unwrap(),
        Replay::from_bytes(&bytes).unwrap()
    );
}

#[test]
fn reader_failure_inside_score_length_is_not_treated_as_missing_metadata() {
    struct FailsAfter<'a>(&'a [u8]);
    impl std::io::Read for FailsAfter<'_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            if self.0.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "injected read failure",
                ));
            }
            std::io::Read::read(&mut self.0, out)
        }
    }
    let bytes = fixture(0, 30000001, -1, "16|256|192|1", &[1, 2]);
    let err = Replay::from_reader(FailsAfter(&bytes)).unwrap_err();
    assert!(
        matches!(err, rosu_replay::ReplayError::Io(e) if e.kind() == std::io::ErrorKind::PermissionDenied)
    );
}
